mod token_auth_common;

use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::time::sleep;
use tokio_tungstenite::tungstenite::http::header::AUTHORIZATION;

use token_auth_common::*;

#[derive(Clone, Default)]
struct LogBuffer {
    bytes: Arc<Mutex<Vec<u8>>>,
}

impl LogBuffer {
    fn clear(&self) {
        self.bytes.lock().unwrap().clear();
    }

    fn contents(&self) -> String {
        String::from_utf8_lossy(&self.bytes.lock().unwrap()).to_string()
    }
}

impl Write for LogBuffer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.bytes.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn assert_no_credentials(logs: &str) {
    assert!(!logs.contains(TOKEN_A), "logs contain token A:\n{logs}");
    assert!(!logs.contains(TOKEN_B), "logs contain token B:\n{logs}");
    assert!(
        !logs.contains("Authorization:"),
        "logs contain a full Authorization header:\n{logs}"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_s01_s02_s03_s04_logs_errors_and_responses_do_not_leak_credentials() {
    let logger = LogBuffer::default();
    let writer = logger.clone();
    // Capture INFO and above only: tungstenite TRACE logs print the raw handshake request,
    // including the Authorization header. That third-party TRACE should not be enabled in
    // production either.
    tracing_subscriber::fmt()
        .with_writer(move || writer.clone())
        .with_ansi(false)
        .with_max_level(tracing::Level::INFO)
        .init();

    // S-01: server rejection logs.
    let server = start_server(Some(TOKEN_A), Vec::new).await;
    for headers in [
        vec![],
        vec![("Authorization", format!("Bearer {TOKEN_B}"))],
        vec![("Authorization", "Basic test-token-A_2026".to_string())],
        vec![("Authorization", "Bearer".to_string())],
    ] {
        let borrowed: Vec<(&str, &str)> = headers
            .iter()
            .map(|(name, value)| (*name, value.as_str()))
            .collect();
        expect_http_status(connect_ws(server.port, "/onebot", &borrowed).await, 401);
    }
    sleep(Duration::from_millis(200)).await;
    let server_logs = logger.contents();
    assert!(
        server_logs.contains("reverse WebSocket handshake rejected"),
        "expected rejection log, got: {server_logs}"
    );
    assert!(
        server_logs.contains("reverse WebSocket handshake failed"),
        "expected outer handshake failure log, got: {server_logs}"
    );
    assert_no_credentials(&server_logs);
    logger.clear();

    // S-02: client error logs, including upstream reflection of the request header.
    for status in [401u16, 403, 500] {
        logger.clear();
        let upstream = start_upstream(UpstreamConfig {
            actions: vec![HandshakeAction::Reject(status)],
            reflect_auth: true,
            ..Default::default()
        })
        .await;
        let client = start_client(&ws_url(upstream.port, "/onebot"), Some(TOKEN_A), Vec::new());
        upstream.wait_for_handshakes(1).await;
        sleep(Duration::from_millis(200)).await;
        let client_logs = logger.contents();
        assert!(
            client_logs.contains("connect error"),
            "expected client connect error log for {status}, got: {client_logs}"
        );
        assert_no_credentials(&client_logs);
        assert!(client.stop().await.is_ok());
        upstream.stop().await;
    }

    // S-03: request/header Debug output.
    let request = request_with_headers(
        &ws_url(1, "/onebot"),
        &[("Authorization", &format!("Bearer {TOKEN_A}"))],
    );
    let request_debug = format!("{request:?}");
    assert!(!request_debug.contains(TOKEN_A));
    let header_debug = format!("{:?}", request.headers().get(AUTHORIZATION).unwrap());
    assert!(!header_debug.contains(TOKEN_A));

    // S-04: different invalid credentials must produce the same fixed response.
    let mut signature: Option<(u16, String, String, String, String, Vec<u8>)> = None;
    for headers in [
        vec![],
        vec![("Authorization", format!("Bearer {TOKEN_B}"))],
        vec![("Authorization", "Basic wrong".to_string())],
        vec![("Authorization", "Bearer".to_string())],
    ] {
        let borrowed: Vec<(&str, &str)> = headers
            .iter()
            .map(|(name, value)| (*name, value.as_str()))
            .collect();
        let response = parse_raw_response(
            &raw_request(server.port, &raw_ws_request("/onebot", &borrowed)).await,
        );
        let current = (
            response.status,
            response
                .header("WWW-Authenticate")
                .unwrap_or("")
                .to_string(),
            response.header("Content-Type").unwrap_or("").to_string(),
            response.header("Content-Length").unwrap_or("").to_string(),
            response.header("Connection").unwrap_or("").to_string(),
            response.body.clone(),
        );
        match &signature {
            None => {
                assert_eq!(current.0, 401);
                assert_eq!(current.5, b"Unauthorized");
                signature = Some(current);
            }
            Some(expected) => assert_eq!(&current, expected),
        }
    }

    server.stop().await;
}
