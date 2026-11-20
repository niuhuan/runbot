#![allow(dead_code)]
#![allow(clippy::result_large_err)]

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use runbot::prelude::*;
use serde_json::{Value, json};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Notify;
use tokio::task::JoinHandle;
use tokio::time::{sleep, timeout};
use tokio_tungstenite::tungstenite::Error as WsError;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::handshake::server::{
    ErrorResponse, Request as HandshakeRequest, Response as HandshakeResponse,
};
use tokio_tungstenite::tungstenite::http::{
    HeaderName, HeaderValue, StatusCode, header::AUTHORIZATION,
};
use tokio_tungstenite::tungstenite::protocol::Message as WsMessage;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, accept_hdr_async, connect_async};

pub const TOKEN_A: &str = "test-token-A_2026";
pub const TOKEN_B: &str = "test-token-B_2026";

pub fn ws_url(port: u16, path: &str) -> String {
    format!("ws://127.0.0.1:{port}{path}")
}

pub fn message_event(marker: &str) -> String {
    json!({
        "post_type": "message",
        "message_type": "private",
        "sub_type": "friend",
        "self_id": 1,
        "user_id": 10001,
        "time": 0,
        "message_id": 1,
        "message_seq": 1,
        "sender": {
            "user_id": 10001,
            "nickname": "tester"
        },
        "raw_message": marker,
        "font": 0,
        "message": [
            {
                "type": "text",
                "data": {
                    "text": marker
                }
            }
        ],
        "message_format": "array"
    })
    .to_string()
}

pub fn action_response(echo: &str, message_id: i64) -> String {
    json!({
        "status": "ok",
        "retcode": 0,
        "data": {
            "message_id": message_id
        },
        "message": "",
        "wording": "",
        "echo": echo
    })
    .to_string()
}

#[derive(Debug)]
pub struct CountingProcessor {
    pub messages: Arc<AtomicUsize>,
    pub api_ok: Arc<AtomicUsize>,
    pub trigger: &'static str,
}

#[async_trait::async_trait]
impl MessageProcessor for CountingProcessor {
    fn id(&self) -> &'static str {
        "token_auth_test_processor"
    }

    async fn process_message(
        &self,
        bot_ctx: Arc<BotContext>,
        message: &Message,
    ) -> anyhow::Result<bool> {
        self.messages.fetch_add(1, Ordering::SeqCst);
        if message.raw_message == self.trigger {
            let response = bot_ctx
                .send_private_message(message.user_id, "reply")
                .await?;
            let data = response
                .wait_response_with_timeout(Duration::from_secs(3))
                .await?;
            if data.message_id == 12345 {
                self.api_ok.fetch_add(1, Ordering::SeqCst);
            }
        }
        Ok(true)
    }
}

pub fn counting_processor(
    messages: Arc<AtomicUsize>,
    api_ok: Arc<AtomicUsize>,
    trigger: &'static str,
) -> Processor {
    let processor = CountingProcessor {
        messages,
        api_ok,
        trigger,
    };
    (Box::new(processor) as Box<dyn MessageProcessor>).into()
}

pub async fn free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    port
}

pub async fn wait_for_listener(port: u16) -> bool {
    timeout(Duration::from_millis(200), async {
        loop {
            if TcpStream::connect(("127.0.0.1", port)).await.is_ok() {
                return true;
            }
            sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap_or(false)
}

pub struct ServerHarness {
    pub port: u16,
    task: JoinHandle<()>,
}

impl ServerHarness {
    pub async fn stop(self) {
        self.task.abort();
        let _ = self.task.await;
    }

    pub fn is_running(&self) -> bool {
        !self.task.is_finished()
    }
}

pub async fn start_server<F>(access_token: Option<&str>, make_processors: F) -> ServerHarness
where
    F: Fn() -> Vec<Processor>,
{
    for _ in 0..20 {
        let port = free_port().await;
        let mut builder = BotServerBuilder::new().bind(format!("127.0.0.1:{port}"));
        if let Some(token) = access_token {
            builder = builder.access_token(token);
        }
        for processor in make_processors() {
            builder = builder.add_processor(processor);
        }
        let server = match builder.build() {
            Ok(server) => server,
            Err(_) => continue,
        };
        let task = tokio::spawn(async move {
            let _ = loop_server(server).await;
        });
        if wait_for_listener(port).await {
            return ServerHarness { port, task };
        }
        task.abort();
        let _ = task.await;
    }
    panic!("failed to start test server");
}

pub fn raw_ws_request(path: &str, headers: &[(&str, &str)]) -> Vec<u8> {
    let mut request = format!(
        "GET {path} HTTP/1.1\r\n\
         Host: 127.0.0.1\r\n\
         Connection: Upgrade\r\n\
         Upgrade: websocket\r\n\
         Sec-WebSocket-Version: 13\r\n\
         Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n"
    );
    for (name, value) in headers {
        request.push_str(name);
        request.push_str(": ");
        request.push_str(value);
        request.push_str("\r\n");
    }
    request.push_str("\r\n");
    request.into_bytes()
}

#[derive(Debug, Clone)]
pub struct RawResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl RawResponse {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }
}

pub async fn raw_request(port: u16, request: &[u8]) -> Vec<u8> {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
    stream.write_all(request).await.unwrap();
    let mut response = Vec::new();
    let _ = timeout(Duration::from_secs(3), stream.read_to_end(&mut response)).await;
    response
}

pub async fn raw_request_or_timeout(port: u16, request: &[u8]) -> Vec<u8> {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
    stream.write_all(request).await.unwrap();
    let mut response = Vec::new();
    let _ = timeout(
        Duration::from_millis(500),
        stream.read_to_end(&mut response),
    )
    .await;
    response
}

pub fn parse_raw_response(bytes: &[u8]) -> RawResponse {
    let split = bytes
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .expect("response should contain header terminator");
    let head = std::str::from_utf8(&bytes[..split]).expect("response head should be UTF-8");
    let body = bytes[split + 4..].to_vec();
    let mut lines = head.split("\r\n");
    let status_line = lines.next().expect("response should have status line");
    let status = status_line
        .split_whitespace()
        .nth(1)
        .expect("status line should have status code")
        .parse()
        .expect("status code should be numeric");
    let headers = lines
        .filter_map(|line| line.split_once(':'))
        .map(|(name, value)| (name.trim().to_ascii_lowercase(), value.trim().to_string()))
        .collect();
    RawResponse {
        status,
        headers,
        body,
    }
}

pub fn request_with_headers(url: &str, headers: &[(&str, &str)]) -> HandshakeRequest {
    let mut request = url.into_client_request().unwrap();
    for (name, value) in headers {
        let name = HeaderName::from_bytes(name.as_bytes()).unwrap();
        let mut value = HeaderValue::from_str(value).unwrap();
        if name == AUTHORIZATION {
            value.set_sensitive(true);
        }
        request.headers_mut().append(name, value);
    }
    request
}

pub async fn connect_ws(
    port: u16,
    path: &str,
    headers: &[(&str, &str)],
) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, WsError> {
    let url = ws_url(port, path);
    let (ws, _) = connect_async(request_with_headers(&url, headers)).await?;
    Ok(ws)
}

pub fn expect_http_status(
    result: Result<WebSocketStream<MaybeTlsStream<TcpStream>>, WsError>,
    status: u16,
) {
    match result {
        Err(WsError::Http(response)) => {
            assert_eq!(
                response.status(),
                StatusCode::from_u16(status).unwrap(),
                "unexpected HTTP status"
            );
        }
        Err(error) => panic!("expected HTTP {status}, got {error}"),
        Ok(_) => panic!("expected HTTP {status}, got successful WebSocket"),
    }
}

pub async fn send_text(ws: &mut WebSocketStream<MaybeTlsStream<TcpStream>>, text: String) {
    ws.send(WsMessage::Text(text.into())).await.unwrap();
}

pub async fn recv_text(ws: &mut WebSocketStream<MaybeTlsStream<TcpStream>>) -> String {
    let message = timeout(Duration::from_secs(3), ws.next())
        .await
        .expect("timed out waiting for WebSocket message")
        .expect("WebSocket stream ended")
        .expect("WebSocket message error");
    match message {
        WsMessage::Text(text) => text.to_string(),
        other => panic!("expected text message, got {other:?}"),
    }
}

#[derive(Clone, Copy, Debug)]
pub enum HandshakeAction {
    Accept,
    Reject(u16),
}

#[derive(Clone, Debug, Default)]
pub struct UpstreamConfig {
    pub actions: Vec<HandshakeAction>,
    pub send_event: bool,
    pub reflect_auth: bool,
    pub close_after_first_accept: bool,
}

#[derive(Debug, Clone)]
pub struct RecordedHandshake {
    pub authorization: Option<String>,
    pub authorization_count: usize,
    pub path: String,
    pub query: Option<String>,
}

pub struct UpstreamHarness {
    pub port: u16,
    pub records: Arc<Mutex<Vec<RecordedHandshake>>>,
    task: JoinHandle<()>,
}

impl UpstreamHarness {
    pub async fn stop(self) {
        self.task.abort();
        let _ = self.task.await;
    }

    pub fn handshakes(&self) -> Vec<RecordedHandshake> {
        self.records.lock().unwrap().clone()
    }

    pub async fn wait_for_handshakes(&self, count: usize) -> Vec<RecordedHandshake> {
        self.wait_for_handshakes_timeout(count, Duration::from_secs(5))
            .await
    }

    pub async fn wait_for_handshakes_timeout(
        &self,
        count: usize,
        timeout_after: Duration,
    ) -> Vec<RecordedHandshake> {
        timeout(timeout_after, async {
            loop {
                let records = self.handshakes();
                if records.len() >= count {
                    return records;
                }
                sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .unwrap_or_else(|_| panic!("timed out waiting for {count} handshakes"))
    }
}

pub async fn start_upstream(config: UpstreamConfig) -> UpstreamHarness {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    start_upstream_with_listener(listener, port, config).await
}

pub async fn start_upstream_on_port(port: u16, config: UpstreamConfig) -> UpstreamHarness {
    let listener = TcpListener::bind(("127.0.0.1", port)).await.unwrap();
    start_upstream_with_listener(listener, port, config).await
}

async fn start_upstream_with_listener(
    listener: TcpListener,
    port: u16,
    config: UpstreamConfig,
) -> UpstreamHarness {
    let records = Arc::new(Mutex::new(Vec::new()));
    let task_records = records.clone();
    let task = tokio::spawn(async move {
        let accepted = Arc::new(AtomicUsize::new(0));
        let mut connection_index = 0usize;
        loop {
            let Ok((stream, _)) = listener.accept().await else {
                break;
            };
            let action = config
                .actions
                .get(connection_index)
                .copied()
                .unwrap_or(HandshakeAction::Accept);
            connection_index += 1;
            let records = task_records.clone();
            let accepted = accepted.clone();
            let send_event = config.send_event;
            let reflect_auth = config.reflect_auth;
            let close_first = config.close_after_first_accept;
            tokio::spawn(async move {
                let auth_for_callback = Arc::new(Mutex::new(None::<String>));
                let auth_for_callback_inner = auth_for_callback.clone();
                let result = accept_hdr_async(
                    stream,
                    move |request: &HandshakeRequest, response: HandshakeResponse| {
                        let values: Vec<String> = request
                            .headers()
                            .get_all(AUTHORIZATION)
                            .iter()
                            .filter_map(|value| value.to_str().ok().map(str::to_owned))
                            .collect();
                        let authorization = values.first().cloned();
                        *auth_for_callback_inner.lock().unwrap() = authorization.clone();
                        records.lock().unwrap().push(RecordedHandshake {
                            authorization: authorization.clone(),
                            authorization_count: values.len(),
                            path: request.uri().path().to_string(),
                            query: request.uri().query().map(str::to_owned),
                        });
                        match action {
                            HandshakeAction::Accept => Ok(response),
                            HandshakeAction::Reject(status) => {
                                let body = if reflect_auth {
                                    format!("denied {authorization:?}")
                                } else {
                                    "denied".to_string()
                                };
                                let mut response = ErrorResponse::new(Some(body));
                                *response.status_mut() = StatusCode::from_u16(status).unwrap();
                                if reflect_auth && let Some(authorization) = authorization {
                                    response.headers_mut().insert(
                                        AUTHORIZATION,
                                        HeaderValue::from_str(&authorization).unwrap(),
                                    );
                                }
                                Err(response)
                            }
                        }
                    },
                )
                .await;

                let Ok(ws) = result else {
                    return;
                };
                let accepted_index = accepted.fetch_add(1, Ordering::SeqCst);
                if close_first && accepted_index == 0 {
                    sleep(Duration::from_millis(100)).await;
                    return;
                }
                if send_event {
                    handle_upstream_connection(ws).await;
                } else {
                    wait_for_close(ws).await;
                }
            });
        }
    });
    UpstreamHarness {
        port,
        records,
        task,
    }
}

async fn handle_upstream_connection<S>(ws: WebSocketStream<S>)
where
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    let (mut sink, mut stream) = ws.split();
    let _ = sink
        .send(WsMessage::Text(message_event("trigger-api").into()))
        .await;
    while let Some(message) = stream.next().await {
        let Ok(WsMessage::Text(text)) = message else {
            continue;
        };
        let Ok(value) = serde_json::from_str::<Value>(&text) else {
            continue;
        };
        let Some(echo) = value.get("echo").and_then(Value::as_str) else {
            continue;
        };
        let response = action_response(echo, 12345);
        if sink.send(WsMessage::Text(response.into())).await.is_err() {
            break;
        }
    }
}

async fn wait_for_close<S>(mut ws: WebSocketStream<S>)
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    while let Some(Ok(_)) = ws.next().await {}
}

pub struct ClientHarness {
    pub bot_ctx: Arc<BotContext>,
    task: JoinHandle<runbot::error::Result<()>>,
}

impl ClientHarness {
    pub async fn stop(self) -> runbot::error::Result<()> {
        self.bot_ctx.shutdown().await?;
        timeout(Duration::from_secs(2), self.task)
            .await
            .expect("client task did not stop within 2 seconds")
            .expect("client task join error")
    }

    pub fn is_running(&self) -> bool {
        !self.task.is_finished()
    }
}

pub fn start_client(
    url: &str,
    access_token: Option<&str>,
    processors: Vec<Processor>,
) -> ClientHarness {
    let mut builder = BotContextBuilder::new().url(url);
    if let Some(token) = access_token {
        builder = builder.access_token(token);
    }
    for processor in processors {
        builder = builder.add_processor(processor);
    }
    let bot_ctx = builder.build().unwrap();
    let task_ctx = bot_ctx.clone();
    let task = tokio::spawn(async move { loop_client(task_ctx).await });
    ClientHarness { bot_ctx, task }
}

pub struct StallUpstream {
    pub port: u16,
    pub accepted: Arc<Notify>,
    task: JoinHandle<()>,
}

impl StallUpstream {
    pub async fn stop(self) {
        self.task.abort();
        let _ = self.task.await;
    }
}

pub async fn start_stall_upstream() -> StallUpstream {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let accepted = Arc::new(Notify::new());
    let task_accepted = accepted.clone();
    let task = tokio::spawn(async move {
        loop {
            let Ok((stream, _)) = listener.accept().await else {
                break;
            };
            task_accepted.notify_one();
            tokio::spawn(async move {
                let _stream = stream;
                sleep(Duration::from_secs(30)).await;
            });
        }
    });
    StallUpstream {
        port,
        accepted,
        task,
    }
}

pub async fn wait_for_condition<F>(timeout_after: Duration, mut condition: F)
where
    F: FnMut() -> bool,
{
    timeout(timeout_after, async {
        while !condition() {
            sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("condition was not met before timeout");
}

pub fn source_has_handshake_before_context() -> bool {
    let source = include_str!("../../src/connection.rs");
    let handshake = source
        .find("accept_websocket(stream")
        .expect("handshake call should exist");
    let context = source
        .find("Arc::new(BotContext")
        .expect("context creation should exist");
    handshake < context
}
