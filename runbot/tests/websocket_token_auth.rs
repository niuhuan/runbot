mod token_auth_common;

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use futures_util::SinkExt;
use runbot::prelude::*;
use serde_json::Value;
use tokio::time::sleep;
use tokio_tungstenite::tungstenite::protocol::Message as WsMessage;

use token_auth_common::*;

fn new_counter() -> Arc<AtomicUsize> {
    Arc::new(AtomicUsize::new(0))
}

// ---------------------------------------------------------------------------
// Server: handshake and real accept loop (I-S01 ... I-S10)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_i_s01_server_without_token_accepts_all_clients() {
    let server = start_server(None, Vec::new).await;
    let cases: [&[(&str, &str)]; 3] = [
        &[],
        &[("Authorization", "Bearer test-token-B_2026")],
        &[("Authorization", "Basic anything")],
    ];
    for headers in cases {
        let mut ws = connect_ws(server.port, "/onebot?scene=test", headers)
            .await
            .expect("server without token should accept the connection");
        ws.close(None).await.ok();
    }
    server.stop().await;
}

#[tokio::test]
async fn test_i_s02_server_accepts_matching_token_and_processes_message() {
    let messages = new_counter();
    let api_ok = new_counter();
    let processor_messages = messages.clone();
    let processor_api = api_ok.clone();
    let server = start_server(Some(TOKEN_A), move || {
        vec![counting_processor(
            processor_messages.clone(),
            processor_api.clone(),
            "never-trigger-api",
        )]
    })
    .await;

    for (name, value) in [
        ("Authorization", format!("Bearer {TOKEN_A}")),
        ("authorization", format!("bEaReR {TOKEN_A}")),
    ] {
        let mut ws = connect_ws(server.port, "/onebot", &[(name, value.as_str())])
            .await
            .expect("matching token should be accepted");
        send_text(&mut ws, message_event("hello")).await;
        ws.close(None).await.ok();
    }

    wait_for_condition(Duration::from_secs(3), || {
        messages.load(Ordering::SeqCst) >= 2
    })
    .await;
    server.stop().await;
}

#[tokio::test]
async fn test_i_s03_server_rejects_invalid_credentials() {
    let messages = new_counter();
    let api_ok = new_counter();
    let processor_messages = messages.clone();
    let processor_api = api_ok.clone();
    let server = start_server(Some(TOKEN_A), move || {
        vec![counting_processor(
            processor_messages.clone(),
            processor_api.clone(),
            "never-trigger-api",
        )]
    })
    .await;

    let cases: [&[(&str, &str)]; 6] = [
        &[],
        &[("Authorization", "Bearer test-token-B_2026")],
        &[("Authorization", "Bearer TEST-TOKEN-A_2026")],
        &[("Authorization", "Basic test-token-A_2026")],
        &[("Authorization", "Bearer ")],
        &[("Authorization", "Bearer")],
    ];
    for headers in cases {
        expect_http_status(connect_ws(server.port, "/onebot", headers).await, 401);
    }
    sleep(Duration::from_millis(100)).await;
    assert_eq!(messages.load(Ordering::SeqCst), 0);
    server.stop().await;
}

#[tokio::test]
async fn test_i_s04_server_does_not_accept_token_from_query_header_or_first_message() {
    let messages = new_counter();
    let api_ok = new_counter();
    let processor_messages = messages.clone();
    let processor_api = api_ok.clone();
    let server = start_server(Some(TOKEN_A), move || {
        vec![counting_processor(
            processor_messages.clone(),
            processor_api.clone(),
            "never-trigger-api",
        )]
    })
    .await;

    expect_http_status(
        connect_ws(server.port, &format!("/onebot?access_token={TOKEN_A}"), &[]).await,
        401,
    );
    expect_http_status(
        connect_ws(server.port, "/onebot", &[("X-Access-Token", TOKEN_A)]).await,
        401,
    );
    expect_http_status(connect_ws(server.port, "/onebot", &[]).await, 401);
    sleep(Duration::from_millis(100)).await;
    assert_eq!(messages.load(Ordering::SeqCst), 0);
    server.stop().await;
}

#[tokio::test]
async fn test_i_s05_rejection_response_is_complete_and_connection_closes() {
    let server = start_server(Some(TOKEN_A), Vec::new).await;
    let cases: [&[(&str, &str)]; 4] = [
        &[],
        &[("Authorization", "Bearer test-token-B_2026")],
        &[("Authorization", "Basic test-token-A_2026")],
        &[("Authorization", "Bearer ")],
    ];
    for headers in cases {
        let bytes = raw_request(server.port, &raw_ws_request("/onebot", headers)).await;
        let response = parse_raw_response(&bytes);
        assert_eq!(response.status, 401);
        assert_eq!(response.body, b"Unauthorized");
        assert_eq!(response.header("WWW-Authenticate"), Some("Bearer"));
        assert_eq!(
            response.header("Content-Type"),
            Some("text/plain; charset=utf-8")
        );
        assert_eq!(response.header("Content-Length"), Some("12"));
        assert_eq!(response.header("Connection"), Some("close"));
        assert!(!String::from_utf8_lossy(&bytes).contains("101 Switching Protocols"));
    }
    server.stop().await;
}

#[tokio::test]
async fn test_i_s06_malformed_handshakes_do_not_stop_accept_loop() {
    let messages = new_counter();
    let api_ok = new_counter();
    let processor_messages = messages.clone();
    let processor_api = api_ok.clone();
    let server = start_server(Some(TOKEN_A), move || {
        vec![counting_processor(
            processor_messages.clone(),
            processor_api.clone(),
            "never-trigger-api",
        )]
    })
    .await;

    let _ = raw_request_or_timeout(server.port, b"BADREQUEST\r\n\r\n").await;
    let _ = raw_request_or_timeout(server.port, b"GET / HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n").await;
    let _ = raw_request_or_timeout(server.port, b"GET / HTTP/1.1\r\nHost: 127.0.0.1\r\n").await;
    assert!(server.is_running());

    let mut ws = connect_ws(
        server.port,
        "/onebot",
        &[("Authorization", &format!("Bearer {TOKEN_A}"))],
    )
    .await
    .expect("valid handshake should still succeed");
    send_text(&mut ws, message_event("after-malformed")).await;
    wait_for_condition(Duration::from_secs(3), || {
        messages.load(Ordering::SeqCst) >= 1
    })
    .await;
    ws.close(None).await.ok();
    server.stop().await;
}

#[tokio::test]
async fn test_i_s07_repeated_rejections_then_success() {
    let messages = new_counter();
    let api_ok = new_counter();
    let processor_messages = messages.clone();
    let processor_api = api_ok.clone();
    let server = start_server(Some(TOKEN_A), move || {
        vec![counting_processor(
            processor_messages.clone(),
            processor_api.clone(),
            "never-trigger-api",
        )]
    })
    .await;

    for index in 0..20 {
        let expected = if index % 2 == 0 {
            TOKEN_B
        } else {
            "wrong-token"
        };
        expect_http_status(
            connect_ws(
                server.port,
                "/onebot",
                &[("Authorization", &format!("Bearer {expected}"))],
            )
            .await,
            401,
        );
    }
    let mut ws = connect_ws(
        server.port,
        "/onebot",
        &[("Authorization", &format!("Bearer {TOKEN_A}"))],
    )
    .await
    .expect("valid request after rejections should succeed");
    send_text(&mut ws, message_event("after-rejections")).await;
    wait_for_condition(Duration::from_secs(3), || {
        messages.load(Ordering::SeqCst) >= 1
    })
    .await;
    ws.close(None).await.ok();
    server.stop().await;
}

#[tokio::test]
async fn test_i_s08_concurrent_valid_and_invalid_connections_are_isolated() {
    let messages = new_counter();
    let api_ok = new_counter();
    let processor_messages = messages.clone();
    let processor_api = api_ok.clone();
    let server = start_server(Some(TOKEN_A), move || {
        vec![counting_processor(
            processor_messages.clone(),
            processor_api.clone(),
            "never-trigger-api",
        )]
    })
    .await;

    let mut tasks = Vec::new();
    for index in 0..10 {
        let port = server.port;
        tasks.push(tokio::spawn(async move {
            let mut ws = connect_ws(
                port,
                "/onebot",
                &[("Authorization", &format!("Bearer {TOKEN_A}"))],
            )
            .await
            .expect("valid connection should succeed");
            send_text(&mut ws, message_event(&format!("valid-{index}"))).await;
            sleep(Duration::from_millis(50)).await;
            ws.close(None).await.ok();
        }));
    }
    for _ in 0..10 {
        let port = server.port;
        tasks.push(tokio::spawn(async move {
            expect_http_status(
                connect_ws(
                    port,
                    "/onebot",
                    &[("Authorization", &format!("Bearer {TOKEN_B}"))],
                )
                .await,
                401,
            );
        }));
    }
    for task in tasks {
        task.await.expect("connection task should not panic");
    }

    wait_for_condition(Duration::from_secs(3), || {
        messages.load(Ordering::SeqCst) >= 10
    })
    .await;
    server.stop().await;
}

#[tokio::test]
async fn test_i_s09_rejected_connections_do_not_reach_processors_or_context_creation() {
    assert!(
        source_has_handshake_before_context(),
        "source must create BotContext only after handshake"
    );
    let messages = new_counter();
    let api_ok = new_counter();
    let processor_messages = messages.clone();
    let processor_api = api_ok.clone();
    let server = start_server(Some(TOKEN_A), move || {
        vec![counting_processor(
            processor_messages.clone(),
            processor_api.clone(),
            "never-trigger-api",
        )]
    })
    .await;

    let response = raw_request(server.port, &raw_ws_request("/onebot", &[])).await;
    assert_eq!(parse_raw_response(&response).status, 401);
    sleep(Duration::from_millis(100)).await;
    assert_eq!(messages.load(Ordering::SeqCst), 0);

    let mut ws = connect_ws(
        server.port,
        "/onebot",
        &[("Authorization", &format!("Bearer {TOKEN_A}"))],
    )
    .await
    .expect("valid connection should succeed");
    send_text(&mut ws, message_event("after-rejection")).await;
    wait_for_condition(Duration::from_secs(3), || {
        messages.load(Ordering::SeqCst) >= 1
    })
    .await;
    ws.close(None).await.ok();
    server.stop().await;
}

#[tokio::test]
async fn test_i_s10_authenticated_business_and_api_echo_flow() {
    let messages = new_counter();
    let api_ok = new_counter();
    let processor_messages = messages.clone();
    let processor_api = api_ok.clone();
    let server = start_server(Some(TOKEN_A), move || {
        vec![counting_processor(
            processor_messages.clone(),
            processor_api.clone(),
            "trigger-api",
        )]
    })
    .await;

    let mut ws = connect_ws(
        server.port,
        "/onebot",
        &[("Authorization", &format!("Bearer {TOKEN_A}"))],
    )
    .await
    .expect("valid connection should succeed");
    send_text(&mut ws, message_event("trigger-api")).await;

    let action = recv_text(&mut ws).await;
    let action: Value = serde_json::from_str(&action).unwrap();
    assert_eq!(action["action"], "send_private_msg");
    assert_eq!(action["params"]["user_id"], 10001);
    let echo = action["echo"].as_str().unwrap();
    ws.send(WsMessage::Text(action_response(echo, 12345).into()))
        .await
        .unwrap();

    wait_for_condition(Duration::from_secs(3), || {
        api_ok.load(Ordering::SeqCst) >= 1 && messages.load(Ordering::SeqCst) >= 1
    })
    .await;
    ws.close(None).await.ok();
    server.stop().await;
}

#[tokio::test]
async fn test_u_b04_last_client_token_is_used_on_the_wire() {
    let upstream = start_upstream(UpstreamConfig {
        actions: vec![HandshakeAction::Accept],
        ..Default::default()
    })
    .await;
    let bot_ctx = BotContextBuilder::new()
        .url(ws_url(upstream.port, "/onebot"))
        .access_token("token-A")
        .access_token("token-B")
        .build()
        .unwrap();
    let task_ctx = bot_ctx.clone();
    let task = tokio::spawn(async move { loop_client(task_ctx).await });
    let records = upstream.wait_for_handshakes(1).await;
    assert_eq!(records[0].authorization.as_deref(), Some("Bearer token-B"));
    bot_ctx.shutdown().await.unwrap();
    let result = tokio::time::timeout(Duration::from_secs(2), task)
        .await
        .expect("client should stop")
        .expect("client task should not panic");
    assert!(result.is_ok());
    upstream.stop().await;
}

#[tokio::test]
async fn test_u_b04_last_server_token_is_used_on_the_wire() {
    let port = free_port().await;
    let server = BotServerBuilder::new()
        .bind(format!("127.0.0.1:{port}"))
        .access_token("token-A")
        .access_token("token-B")
        .build()
        .unwrap();
    let task = tokio::spawn(async move {
        let _ = loop_server(server).await;
    });
    assert!(wait_for_listener(port).await);
    let mut ws = connect_ws(port, "/onebot", &[("Authorization", "Bearer token-B")])
        .await
        .expect("last configured server token should be accepted");
    ws.close(None).await.ok();
    expect_http_status(
        connect_ws(port, "/onebot", &[("Authorization", "Bearer token-A")]).await,
        401,
    );
    task.abort();
    let _ = task.await;
}

// ---------------------------------------------------------------------------
// Client: handshake, reconnect, shutdown and config errors (I-C01 ... I-C07)
// ---------------------------------------------------------------------------

async fn run_client_echo_case(token: Option<&str>) {
    let api_ok = new_counter();
    let messages = new_counter();
    let upstream = start_upstream(UpstreamConfig {
        send_event: true,
        ..Default::default()
    })
    .await;
    let client_api = api_ok.clone();
    let client_messages = messages.clone();
    let client = start_client(
        &ws_url(upstream.port, "/onebot?scene=test"),
        token,
        vec![counting_processor(
            client_messages,
            client_api,
            "trigger-api",
        )],
    );

    let records = upstream.wait_for_handshakes(1).await;
    wait_for_condition(Duration::from_secs(3), || {
        api_ok.load(Ordering::SeqCst) >= 1
    })
    .await;

    assert_eq!(records[0].path, "/onebot");
    assert_eq!(records[0].query.as_deref(), Some("scene=test"));
    match token {
        Some(token) => {
            assert_eq!(records[0].authorization_count, 1);
            assert_eq!(
                records[0].authorization.as_deref(),
                Some(format!("Bearer {token}").as_str())
            );
        }
        None => assert!(records[0].authorization.is_none()),
    }
    assert!(client.stop().await.is_ok());
    upstream.stop().await;
}

#[tokio::test]
async fn test_i_c01_client_sends_no_token_or_exact_bearer_token() {
    run_client_echo_case(None).await;
    run_client_echo_case(Some(TOKEN_A)).await;
}

#[tokio::test]
async fn test_i_c02_client_handshake_matrix() {
    let cases = [
        (None, None, true),
        (None, Some(TOKEN_A), true),
        (Some(TOKEN_A), Some(TOKEN_A), true),
        (Some(TOKEN_A), None, false),
        (Some(TOKEN_A), Some(TOKEN_B), false),
    ];
    for (server_token, client_token, should_accept) in cases {
        let config = if should_accept {
            UpstreamConfig {
                actions: vec![HandshakeAction::Accept],
                send_event: true,
                ..Default::default()
            }
        } else {
            UpstreamConfig {
                actions: vec![HandshakeAction::Reject(401)],
                ..Default::default()
            }
        };
        let upstream = start_upstream(config).await;
        let messages = new_counter();
        let api_ok = new_counter();
        let client = start_client(
            &ws_url(upstream.port, "/onebot"),
            client_token,
            vec![counting_processor(
                messages.clone(),
                api_ok.clone(),
                "trigger-api",
            )],
        );
        let records = upstream.wait_for_handshakes(1).await;
        match (server_token, client_token, should_accept) {
            (None, None, true) => assert!(records[0].authorization.is_none()),
            (None, Some(actual), true) => {
                assert_eq!(
                    records[0].authorization.as_deref(),
                    Some(format!("Bearer {actual}").as_str())
                );
            }
            (Some(expected), Some(actual), true) => {
                assert_eq!(expected, actual);
                assert_eq!(
                    records[0].authorization.as_deref(),
                    Some(format!("Bearer {actual}").as_str())
                );
            }
            (Some(_), _, false) => {
                assert!(
                    records[0].authorization.is_none()
                        || records[0].authorization.as_deref()
                            != Some(format!("Bearer {TOKEN_A}").as_str())
                );
            }
            _ => {}
        }
        if should_accept {
            wait_for_condition(Duration::from_secs(3), || {
                api_ok.load(Ordering::SeqCst) >= 1
            })
            .await;
        } else {
            sleep(Duration::from_millis(100)).await;
            assert_eq!(messages.load(Ordering::SeqCst), 0);
        }
        assert!(client.stop().await.is_ok());
        upstream.stop().await;
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_i_c03_client_retries_after_401_and_403_then_succeeds() {
    let upstream = start_upstream(UpstreamConfig {
        actions: vec![
            HandshakeAction::Reject(401),
            HandshakeAction::Reject(403),
            HandshakeAction::Accept,
        ],
        send_event: true,
        ..Default::default()
    })
    .await;
    let messages = new_counter();
    let api_ok = new_counter();
    let client = start_client(
        &ws_url(upstream.port, "/onebot"),
        Some(TOKEN_A),
        vec![counting_processor(messages, api_ok.clone(), "trigger-api")],
    );

    let records = upstream
        .wait_for_handshakes_timeout(3, Duration::from_secs(45))
        .await;
    wait_for_condition(Duration::from_secs(3), || {
        api_ok.load(Ordering::SeqCst) >= 1
    })
    .await;
    assert_eq!(records.len(), 3);
    for record in records {
        assert_eq!(
            record.authorization.as_deref(),
            Some(format!("Bearer {TOKEN_A}").as_str())
        );
        assert_eq!(record.authorization_count, 1);
    }
    assert!(client.stop().await.is_ok());
    upstream.stop().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_i_c03_client_retries_after_500_then_succeeds() {
    let upstream = start_upstream(UpstreamConfig {
        actions: vec![HandshakeAction::Reject(500), HandshakeAction::Accept],
        send_event: true,
        ..Default::default()
    })
    .await;
    let messages = new_counter();
    let api_ok = new_counter();
    let client = start_client(
        &ws_url(upstream.port, "/onebot"),
        Some(TOKEN_A),
        vec![counting_processor(messages, api_ok.clone(), "trigger-api")],
    );

    upstream
        .wait_for_handshakes_timeout(2, Duration::from_secs(25))
        .await;
    wait_for_condition(Duration::from_secs(3), || {
        api_ok.load(Ordering::SeqCst) >= 1
    })
    .await;
    assert!(client.stop().await.is_ok());
    upstream.stop().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_i_c04_client_reconnects_after_peer_close_with_same_token_and_uri() {
    let upstream = start_upstream(UpstreamConfig {
        actions: vec![HandshakeAction::Accept, HandshakeAction::Accept],
        send_event: true,
        close_after_first_accept: true,
        ..Default::default()
    })
    .await;
    let messages = new_counter();
    let api_ok = new_counter();
    let client = start_client(
        &ws_url(upstream.port, "/onebot?scene=reconnect"),
        Some(TOKEN_A),
        vec![counting_processor(messages, api_ok.clone(), "trigger-api")],
    );

    let records = upstream
        .wait_for_handshakes_timeout(2, Duration::from_secs(25))
        .await;
    wait_for_condition(Duration::from_secs(3), || {
        api_ok.load(Ordering::SeqCst) >= 1
    })
    .await;
    assert!(records.len() >= 2);
    for record in records.into_iter().take(2) {
        assert_eq!(record.path, "/onebot");
        assert_eq!(record.query.as_deref(), Some("scene=reconnect"));
        assert_eq!(
            record.authorization.as_deref(),
            Some(format!("Bearer {TOKEN_A}").as_str())
        );
    }
    assert!(client.stop().await.is_ok());
    upstream.stop().await;
}

#[tokio::test]
async fn test_i_c05_shutdown_during_reconnect_wait() {
    let upstream = start_upstream(UpstreamConfig {
        actions: vec![HandshakeAction::Reject(401)],
        ..Default::default()
    })
    .await;
    let client = start_client(&ws_url(upstream.port, "/onebot"), Some(TOKEN_A), Vec::new());
    upstream.wait_for_handshakes(1).await;
    assert!(client.stop().await.is_ok());
    sleep(Duration::from_millis(200)).await;
    assert_eq!(upstream.handshakes().len(), 1);
    upstream.stop().await;
}

#[tokio::test]
async fn test_i_c05_shutdown_during_stalled_handshake() {
    let upstream = start_stall_upstream().await;
    let client = start_client(&ws_url(upstream.port, "/onebot"), Some(TOKEN_A), Vec::new());
    tokio::time::timeout(Duration::from_secs(2), upstream.accepted.notified())
        .await
        .expect("client should connect to stalled upstream");
    assert!(client.stop().await.is_ok());
    upstream.stop().await;
}

#[tokio::test]
async fn test_i_c05_shutdown_after_connection_is_established() {
    let upstream = start_upstream(UpstreamConfig {
        actions: vec![HandshakeAction::Accept],
        ..Default::default()
    })
    .await;
    let client = start_client(&ws_url(upstream.port, "/onebot"), Some(TOKEN_A), Vec::new());
    upstream.wait_for_handshakes(1).await;
    sleep(Duration::from_millis(100)).await;
    assert!(client.stop().await.is_ok());
    upstream.stop().await;
}

#[tokio::test]
async fn test_i_c06_invalid_client_configuration_exits_without_connecting() {
    for (url, token) in [
        ("ws://127.0.0.1:1", "bad\ntoken"),
        ("not a url", "test-token-A_2026"),
    ] {
        let mut builder = BotContextBuilder::new().url(url);
        builder = builder.access_token(token);
        let bot_ctx = builder.build().unwrap();
        let result = tokio::time::timeout(Duration::from_secs(2), loop_client(bot_ctx))
            .await
            .expect("invalid configuration should return promptly");
        assert!(result.is_err());
        let display = format!("{}", result.unwrap_err());
        assert!(!display.contains(token));
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_i_c07_client_recovers_when_upstream_starts_later() {
    let port = free_port().await;
    let messages = new_counter();
    let api_ok = new_counter();
    let client = start_client(
        &ws_url(port, "/onebot"),
        Some(TOKEN_A),
        vec![counting_processor(messages, api_ok.clone(), "trigger-api")],
    );
    sleep(Duration::from_millis(200)).await;
    let upstream = start_upstream_on_port(
        port,
        UpstreamConfig {
            send_event: true,
            ..Default::default()
        },
    )
    .await;
    let records = upstream
        .wait_for_handshakes_timeout(1, Duration::from_secs(25))
        .await;
    wait_for_condition(Duration::from_secs(3), || {
        api_ok.load(Ordering::SeqCst) >= 1
    })
    .await;
    assert_eq!(
        records[0].authorization.as_deref(),
        Some(format!("Bearer {TOKEN_A}").as_str())
    );
    assert!(client.stop().await.is_ok());
    upstream.stop().await;
}

// ---------------------------------------------------------------------------
// Regression, examples, documentation and exploratory contract checks
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_r02_builder_chains_compile_with_and_without_token() {
    let _server = BotServerBuilder::new()
        .bind("127.0.0.1:3131")
        .access_token(TOKEN_A)
        .build()
        .unwrap();
    let _context = BotContextBuilder::new()
        .url("ws://127.0.0.1:3131")
        .access_token(TOKEN_A)
        .build()
        .unwrap();
}

#[test]
fn test_r01_r05_examples_and_docs_describe_optional_env_token() {
    let client_example = include_str!("../examples/client.rs");
    let server_example = include_str!("../examples/server.rs");
    let readme = include_str!("../../README.md");
    let design = include_str!("../../doc/websocket-token-auth.md");

    for source in [client_example, server_example] {
        assert!(source.contains("RUNBOT_ACCESS_TOKEN"));
        assert!(source.contains("is_empty"));
    }
    for source in [readme, design] {
        assert!(source.contains("access_token"));
        assert!(source.contains("Authorization: Bearer"));
        assert!(source.contains("wss://"));
    }
    assert!(readme.contains("`ws://`"));
}

#[tokio::test]
async fn test_x01_duplicate_authorization_headers_use_first_value() {
    let server = start_server(Some(TOKEN_A), Vec::new).await;
    let cases: [&[(&str, &str)]; 3] = [
        &[
            ("Authorization", "Bearer test-token-A_2026"),
            ("Authorization", "Bearer test-token-B_2026"),
        ],
        &[
            ("Authorization", "Bearer test-token-B_2026"),
            ("Authorization", "Bearer test-token-A_2026"),
        ],
        &[
            ("Authorization", "Bearer test-token-A_2026"),
            ("Authorization", "Bearer test-token-A_2026"),
        ],
    ];
    for headers in cases {
        let result = connect_ws(server.port, "/onebot", headers).await;
        if headers[0].1.ends_with(TOKEN_A) {
            assert!(result.is_ok(), "first correct header should be accepted");
        } else {
            expect_http_status(result, 401);
        }
    }

    let merged = raw_request(
        server.port,
        &raw_ws_request(
            "/onebot",
            &[(
                "Authorization",
                "Bearer test-token-A_2026, Bearer test-token-B_2026",
            )],
        ),
    )
    .await;
    assert_eq!(parse_raw_response(&merged).status, 401);
    server.stop().await;
}

#[tokio::test]
async fn test_x02_whitespace_and_non_ascii_tokens_are_observed_without_panic() {
    let upstream = start_upstream(UpstreamConfig {
        actions: vec![HandshakeAction::Accept],
        ..Default::default()
    })
    .await;
    let client = start_client(&ws_url(upstream.port, "/onebot"), Some("   "), Vec::new());
    let records = upstream.wait_for_handshakes(1).await;
    assert!(records[0].authorization.is_some());
    assert!(client.stop().await.is_ok());
    upstream.stop().await;

    let server = start_server(Some(TOKEN_A), Vec::new).await;
    let response = raw_request(
        server.port,
        &raw_ws_request("/onebot", &[("Authorization", "Bearer test-token-A_2026 ")]),
    )
    .await;
    let parsed = parse_raw_response(&response);
    assert!(
        parsed.status == 101 || parsed.status == 400 || parsed.status == 401,
        "unexpected whitespace-edge status: {}",
        parsed.status
    );
    let valid = connect_ws(
        server.port,
        "/onebot",
        &[("Authorization", "Bearer test-token-A_2026")],
    )
    .await;
    if let Ok(mut ws) = valid {
        ws.close(None).await.ok();
    } else {
        panic!("server should remain usable after whitespace edge input");
    }
    server.stop().await;
}

#[tokio::test]
async fn test_x03_malformed_request_does_not_echo_sensitive_marker_and_server_recovers() {
    let marker = "unique-sensitive-marker_2026";
    let server = start_server(Some(TOKEN_A), Vec::new).await;
    let request = format!(
        "GET / HTTP/1.1\r\n\
         Host: 127.0.0.1\r\n\
         Authorization: Bearer {marker}\r\n\
         X-Broken: {marker}\r\n\
         Bad Header Line Without Colon\r\n\r\n"
    );
    let response = raw_request_or_timeout(server.port, request.as_bytes()).await;
    assert!(
        !String::from_utf8_lossy(&response).contains(marker),
        "protocol error response should not echo the sensitive marker"
    );
    assert!(server.is_running());
    let mut ws = connect_ws(
        server.port,
        "/onebot",
        &[("Authorization", &format!("Bearer {TOKEN_A}"))],
    )
    .await
    .expect("server should recover after malformed request");
    ws.close(None).await.ok();
    server.stop().await;
}

#[test]
fn test_x04_accept_error_backoff_is_present_in_source() {
    let source = include_str!("../src/connection.rs");
    assert!(source.contains("WebSocket accept error"));
    assert!(source.contains("Duration::from_millis(100)"));
}
