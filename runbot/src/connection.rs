//! Forward and reverse WebSocket connection loops.
//!
//! [`loop_client`] connects to the configured upstream and handles reconnection;
//! [`loop_server`] binds an address, accepts inbound connections, and validates
//! `Authorization: Bearer` during the handshake when an access token is configured. Both
//! functions are long-running futures and have no "startup complete, then return" phase.

use std::sync::Arc;

use crate::bot_context::*;
use crate::error::{Error, Result};
use dashmap::DashMap;
use futures_util::{StreamExt, future::Either};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio::time::{Duration, sleep};
use tokio_tungstenite::{
    WebSocketStream, accept_hdr_async, connect_async,
    tungstenite::{
        Error as WsError,
        client::IntoClientRequest,
        handshake::server::{
            ErrorResponse, Request as HandshakeRequest, Response as HandshakeResponse,
        },
        http::{HeaderMap, HeaderValue, StatusCode, header::AUTHORIZATION},
    },
};

const UNAUTHORIZED_BODY: &str = "Unauthorized";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AuthFailure {
    Missing,
    Malformed,
    UnsupportedScheme,
    InvalidToken,
}

impl AuthFailure {
    fn as_str(self) -> &'static str {
        match self {
            Self::Missing => "missing Authorization header",
            Self::Malformed => "malformed Authorization header",
            Self::UnsupportedScheme => "unsupported authentication scheme",
            Self::InvalidToken => "invalid token",
        }
    }
}

/// Compares two byte slices in constant time to avoid timing differences that depend on
/// the first mismatching byte.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    let max_len = a.len().max(b.len());
    let mut diff = a.len() ^ b.len();
    for index in 0..max_len {
        let left = a.get(index).copied().unwrap_or(0);
        let right = b.get(index).copied().unwrap_or(0);
        diff |= usize::from(left ^ right);
    }
    diff == 0
}

fn check_authorization(
    headers: &HeaderMap,
    expected: &str,
) -> std::result::Result<(), AuthFailure> {
    let value = headers
        .get(AUTHORIZATION)
        .ok_or(AuthFailure::Missing)?
        .to_str()
        .map_err(|_| AuthFailure::Malformed)?;
    let (scheme, token) = value.split_once(' ').ok_or(AuthFailure::Malformed)?;
    if !scheme.eq_ignore_ascii_case("Bearer") {
        return Err(AuthFailure::UnsupportedScheme);
    }
    if token.is_empty() {
        return Err(AuthFailure::Malformed);
    }
    if !constant_time_eq(token.as_bytes(), expected.as_bytes()) {
        return Err(AuthFailure::InvalidToken);
    }
    Ok(())
}

fn unauthorized_response() -> ErrorResponse {
    let mut response = ErrorResponse::new(Some(UNAUTHORIZED_BODY.to_string()));
    *response.status_mut() = StatusCode::UNAUTHORIZED;
    let headers = response.headers_mut();
    headers.insert("WWW-Authenticate", HeaderValue::from_static("Bearer"));
    headers.insert(
        "Content-Type",
        HeaderValue::from_static("text/plain; charset=utf-8"),
    );
    headers.insert("Content-Length", HeaderValue::from(UNAUTHORIZED_BODY.len()));
    headers.insert("Connection", HeaderValue::from_static("close"));
    response
}

#[allow(clippy::result_large_err)]
fn authorize_handshake(
    request: &HandshakeRequest,
    response: HandshakeResponse,
    expected: Option<&str>,
) -> std::result::Result<HandshakeResponse, ErrorResponse> {
    let Some(expected) = expected else {
        return Ok(response);
    };
    match check_authorization(request.headers(), expected) {
        Ok(()) => Ok(response),
        Err(reason) => {
            tracing::warn!("reverse WebSocket handshake rejected: {}", reason.as_str());
            Err(unauthorized_response())
        }
    }
}

#[allow(clippy::result_large_err)]
async fn accept_websocket<S>(
    stream: S,
    access_token: Option<String>,
) -> std::result::Result<WebSocketStream<S>, WsError>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    accept_hdr_async(
        stream,
        move |request: &HandshakeRequest, response: HandshakeResponse| {
            authorize_handshake(request, response, access_token.as_deref())
        },
    )
    .await
}

#[allow(clippy::result_large_err)]
fn build_client_request(
    url: &str,
    access_token: Option<&str>,
) -> Result<tokio_tungstenite::tungstenite::http::Request<()>> {
    let mut request = url.into_client_request()?;
    if let Some(token) = access_token {
        if token.is_empty() {
            return Err(Error::ParamsError(
                "access_token must not be empty".to_string(),
            ));
        }
        let mut value = HeaderValue::from_str(&format!("Bearer {token}")).map_err(|_| {
            Error::ParamsError("access_token contains invalid header characters".to_string())
        })?;
        value.set_sensitive(true);
        request.headers_mut().insert(AUTHORIZATION, value);
    }
    Ok(request)
}

async fn loop_bot<S>(bot_ctx: Arc<BotContext>, ws_stream: WebSocketStream<S>)
where
    S: AsyncRead + AsyncWrite + Sync + Send + Unpin + 'static,
{
    // Check whether shutdown has already been requested.
    if bot_ctx.is_shutdown() {
        tracing::warn!("Bot is already shutdown, skipping loop_bot");
        return;
    }

    // Obtain the shutdown receiver.
    let mut shutdown_rx = {
        let rx_lock = bot_ctx.shutdown_rx.lock().await;
        rx_lock.as_ref().cloned()
    };

    let (ws_sink, mut split_stream) = ws_stream.split();
    let connection = BotConnection {
        sender: Box::new(ws_sink),
    };
    bot_ctx.set_connection(connection).await;

    loop {
        // Wait for a message and the shutdown signal concurrently.
        let shutdown_fut = async {
            if let Some(ref mut rx) = shutdown_rx {
                rx.changed().await.ok();
                *rx.borrow()
            } else {
                false
            }
        };

        let msg_fut = split_stream.next();

        match futures_util::future::select(
            Box::pin(shutdown_fut),
            Box::pin(msg_fut),
        ).await {
            Either::Left((shutdown, _)) => {
                if shutdown {
                    tracing::info!("Shutdown signal received, exiting loop_bot");
                    break;
                }
            }
            Either::Right((msg_option, _)) => {
                match msg_option {
                    Some(Ok(m)) => {
                        let bot_ctx = bot_ctx.clone();
                        _ = tokio::spawn(async move { bot_ctx.handle_receive(bot_ctx.clone(), &m).await });
                    }
                    Some(Err(e)) => {
                        tracing::error!("WS error: {:?}", e);
                        break; // Exit the inner loop on disconnect and reconnect.
                    }
                    None => {
                        tracing::info!("WebSocket stream ended");
                        break;
                    }
                }
            }
        }
    }
    bot_ctx.set_connection(None).await;
}

/// Binds the configured address and continuously accepts reverse WebSocket connections.
///
/// This function runs the accept loop; it does not return immediately after startup.
/// After a successful bind, each TCP connection is handled by a separate task that
/// completes the WebSocket handshake. When an access token is configured, only requests
/// carrying a matching `Authorization: Bearer` value are accepted. A failed handshake
/// returns 401 or a protocol error and ends that connection without creating a
/// [`BotContext`], and the accept loop continues serving other connections.
///
/// A connection that completes the handshake gets its own context and message loop. The
/// processor list in `BotServer` is shared by all connections, but the connection, response
/// registry, and shutdown signal are independent per connection.
///
/// # Errors
///
/// Returns [`Error::StateError`] when the address cannot be bound. Runtime accept errors are
/// logged and retried after 100 milliseconds; they do not end this function through its
/// return value. A normally running call never returns.
///
/// # Cancellation
///
/// Cancelling the future ends the accept loop, but connection tasks already spawned are not
/// cancelled or awaited. This function does not provide an overall graceful shutdown API.
///
/// # Examples
///
/// ```no_run
/// use runbot::prelude::*;
///
/// # async fn run() -> runbot::error::Result<()> {
/// let server = BotServerBuilder::new()
///     .bind("0.0.0.0:3131")
///     .build()?;
/// loop_server(server).await
/// # }
/// # fn main() {}
/// ```
pub async fn loop_server(bot_server: Arc<BotServer>) -> Result<()> {
    // Bind the local port.
    let listener = TcpListener::bind(&bot_server.bind)
        .await
        .map_err(|err| Error::StateError(format!("bind {} failed: {err}", bot_server.bind)))?;
    println!("WebSocket server started on ws://{}", &bot_server.bind);
    loop {
        let stream = match listener.accept().await {
            Ok((stream, _)) => stream,
            Err(err) => {
                tracing::error!("WebSocket accept error: {err}");
                sleep(Duration::from_millis(100)).await;
                continue;
            }
        };
        let bot_server = bot_server.clone();
        tokio::spawn(async move {
            // Complete handshake authentication first; do not create a BotContext on failure.
            let ws_stream = match accept_websocket(stream, bot_server.access_token.clone()).await {
                Ok(ws_stream) => ws_stream,
                Err(err) => {
                    tracing::warn!("reverse WebSocket handshake failed: {err}");
                    return;
                }
            };
            let processors = bot_server.processors.clone();
            let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
            loop_bot(
                Arc::new(BotContext {
                    connection: Mutex::new(None),
                    url: None,
                    id: 0,
                    processors,
                    echo_notifer: Arc::new(DashMap::new()),
                    shutdown_tx: Mutex::new(Some(shutdown_tx)),
                    shutdown_rx: Mutex::new(Some(shutdown_rx)),
                    access_token: None,
                }),
                ws_stream,
            )
            .await;
        });
    }
}

/// Connects to the configured forward WebSocket upstream and reconnects after disconnects.
///
/// This function runs the connection loop; it does not return immediately after a
/// connection is established. Every attempt builds a handshake request from
/// [`BotContextBuilder::access_token`]: when a token is configured it sends
/// `Authorization: Bearer <token>`; otherwise the header is omitted. After connecting, it
/// processes events until the connection closes or [`BotContext::shutdown`] is signalled.
///
/// Rejected handshakes, connection failures, and disconnects are logged and retried after
/// 15 seconds. If the shutdown signal arrives during the wait, the loop exits immediately.
/// Configuration errors return directly without entering the reconnect loop. Before a
/// connection is established, API methods that send requests return
/// [`Error::StateError`], so callers should issue business requests only after the
/// connection is ready.
///
/// # Errors
///
/// - Returns [`Error::StateError`] when the context has already been shut down.
/// - Returns [`Error::ParamsError`] when [`BotContextBuilder::url`] is not set.
/// - Returns [`Error::WebSocketError`] when the URL cannot be used to build a request.
/// - Returns [`Error::ParamsError`] when the access token is empty or contains characters
///   that cannot form an HTTP header.
/// - Returns `Ok(())` after a normal shutdown signal.
///
/// Recoverable connection failures are not reported immediately through the return value;
/// they are logged and retried.
///
/// # Cancellation
///
/// Cancelling the future stops the connect/reconnect loop and drops the current local
/// connection; requests already sent to the remote side are not cancelled. Tasks spawned by
/// processors are not guaranteed to finish when this function ends.
///
/// # Examples
///
/// ```no_run
/// use runbot::prelude::*;
///
/// # async fn run() -> runbot::error::Result<()> {
/// let bot_ctx = BotContextBuilder::new()
///     .url("ws://127.0.0.1:3001")
///     .build()?;
/// loop_client(bot_ctx).await
/// # }
/// # fn main() {}
/// ```
pub async fn loop_client(bot_ctx: Arc<BotContext>) -> Result<()> {
    // Check whether shutdown has already been requested.
    if bot_ctx.is_shutdown() {
        return Err(Error::StateError(
            "Bot is already shutdown, cannot start loop_client".to_string(),
        ));
    }

    let url = bot_ctx
        .url
        .as_ref()
        .ok_or(Error::ParamsError(
            "url must be set for loop client".to_string(),
        ))
        .map(|e| e.clone())?;

    // Obtain the shutdown receiver.
    let mut shutdown_rx = {
        let rx_lock = bot_ctx.shutdown_rx.lock().await;
        rx_lock.as_ref().cloned()
    };

    loop {
        // Check the shutdown signal.
        if bot_ctx.is_shutdown() {
            tracing::info!("Shutdown signal received, exiting loop_client");
            return Ok(());
        }

        // Wait for a connection and the shutdown signal concurrently.
        let shutdown_fut = async {
            if let Some(ref mut rx) = shutdown_rx {
                rx.changed().await.ok();
                *rx.borrow()
            } else {
                false
            }
        };

        let request = match build_client_request(&url, bot_ctx.access_token.as_deref()) {
            Ok(request) => request,
            Err(err) => {
                tracing::error!("WS {} request build error: {err}", &url);
                return Err(err);
            }
        };
        let connect_fut = connect_async(request);

        match futures_util::future::select(
            Box::pin(shutdown_fut),
            Box::pin(connect_fut),
        ).await {
            Either::Left((shutdown, _)) => {
                if shutdown {
                    tracing::info!("Shutdown signal received, exiting loop_client");
                    return Ok(());
                }
            }
            Either::Right((connect_result, _)) => {
                match connect_result {
                    Ok((ws_stream, _)) => {
                        tracing::info!("WS {} Connected!", &url);
                        let _ = loop_bot(bot_ctx.clone(), ws_stream).await;
                    }
                    // Use Display only: the Debug form of an HTTP error contains response
                    // headers and body. If the upstream reflects Authorization, Debug could
                    // write credentials into the logs.
                    Err(e) => tracing::error!("WS {} connect error: {}", &url, e),
                }
            }
        }

        // Check the shutdown signal; do not reconnect after shutdown.
        if bot_ctx.is_shutdown() {
            tracing::info!("Shutdown signal received, exiting loop_client");
            return Ok(());
        }

        // Wait for the reconnect delay and the shutdown signal concurrently.
        let shutdown_fut = async {
            if let Some(ref mut rx) = shutdown_rx {
                rx.changed().await.ok();
                *rx.borrow()
            } else {
                false
            }
        };

        let sleep_fut = sleep(Duration::from_secs(15));

        match futures_util::future::select(
            Box::pin(shutdown_fut),
            Box::pin(sleep_fut),
        ).await {
            Either::Left((shutdown, _)) => {
                if shutdown {
                    tracing::info!("Shutdown signal received during reconnect wait, exiting loop_client");
                    return Ok(());
                }
            }
            Either::Right(_) => {
                tracing::info!("WS {} reconnecting after 15s...", &url);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio_tungstenite::tungstenite::http::HeaderName;

    const HANDSHAKE_REQUEST: &str = "GET / HTTP/1.1\r\n\
        Host: localhost\r\n\
        Connection: Upgrade\r\n\
        Upgrade: websocket\r\n\
        Sec-WebSocket-Version: 13\r\n\
        Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n";

    fn authorization_headers(value: Option<&str>) -> HeaderMap {
        let mut headers = HeaderMap::new();
        if let Some(value) = value {
            headers.insert(AUTHORIZATION, HeaderValue::from_str(value).unwrap());
        }
        headers
    }

    #[test]
    fn test_check_authorization_accepts_valid_bearer_token() {
        let headers = authorization_headers(Some("Bearer secret-token"));
        assert_eq!(check_authorization(&headers, "secret-token"), Ok(()));
    }

    #[test]
    fn test_check_authorization_scheme_is_case_insensitive() {
        let headers = authorization_headers(Some("bEaReR secret-token"));
        assert_eq!(check_authorization(&headers, "secret-token"), Ok(()));
    }

    #[test]
    fn test_check_authorization_rejects_missing_header() {
        let headers = authorization_headers(None);
        assert_eq!(
            check_authorization(&headers, "secret-token"),
            Err(AuthFailure::Missing)
        );
    }

    #[test]
    fn test_check_authorization_rejects_malformed_header() {
        for value in [
            "Bearer",
            "Bearer ",
            " Bearer secret-token",
            "Bearer  secret-token",
            "Basic secret-token",
        ] {
            let headers = authorization_headers(Some(value));
            assert!(
                check_authorization(&headers, "secret-token").is_err(),
                "{value:?} should be rejected"
            );
        }
    }

    #[test]
    fn test_check_authorization_rejects_invalid_token() {
        let headers = authorization_headers(Some("Bearer Secret-Token"));
        assert_eq!(
            check_authorization(&headers, "secret-token"),
            Err(AuthFailure::InvalidToken)
        );
    }

    #[test]
    fn test_constant_time_eq() {
        assert!(constant_time_eq(b"secret-token", b"secret-token"));
        assert!(!constant_time_eq(b"secret-token", b"secret-tokeN"));
        assert!(!constant_time_eq(b"secret", b"secret-token"));
        assert!(!constant_time_eq(b"", b"secret-token"));
    }

    #[test]
    fn test_unauthorized_response_does_not_echo_credentials() {
        let response = unauthorized_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(response.body().as_deref(), Some(UNAUTHORIZED_BODY));
        assert_eq!(
            response
                .headers()
                .get("WWW-Authenticate")
                .and_then(|value| value.to_str().ok()),
            Some("Bearer")
        );
    }

    #[test]
    fn test_build_client_request_without_token_has_no_authorization() {
        let request = build_client_request("ws://127.0.0.1:3001", None).unwrap();
        assert!(request.headers().get(AUTHORIZATION).is_none());
    }

    #[test]
    fn test_build_client_request_with_token_sets_sensitive_bearer_header() {
        let request = build_client_request("ws://127.0.0.1:3001", Some("secret-token")).unwrap();
        let value = request.headers().get(AUTHORIZATION).unwrap();
        assert_eq!(value.to_str().unwrap(), "Bearer secret-token");
        assert!(value.is_sensitive());
        assert!(!request.uri().to_string().contains("secret-token"));
    }

    #[test]
    fn test_build_client_request_rejects_invalid_token() {
        assert!(build_client_request("ws://127.0.0.1:3001", Some("")).is_err());
        assert!(build_client_request("ws://127.0.0.1:3001", Some("secret\n-token")).is_err());
    }

    #[test]
    fn test_build_client_request_has_required_upgrade_headers() {
        let request = build_client_request("ws://127.0.0.1:3001", None).unwrap();
        assert_eq!(request.method().as_str(), "GET");
        assert_eq!(request.uri().path(), "/");
        assert_eq!(request.headers().get("Host").unwrap(), "127.0.0.1:3001");
        assert_eq!(request.headers().get("Connection").unwrap(), "Upgrade");
        assert_eq!(request.headers().get("Upgrade").unwrap(), "websocket");
        assert_eq!(
            request.headers().get("Sec-WebSocket-Version").unwrap(),
            "13"
        );
        assert!(
            !request
                .headers()
                .get("Sec-WebSocket-Key")
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn test_build_client_request_preserves_special_token_and_only_one_header() {
        let token = "test-token-A_2026-._~+/=";
        let request = build_client_request("ws://127.0.0.1:3001", Some(token)).unwrap();
        let values: Vec<_> = request.headers().get_all(AUTHORIZATION).iter().collect();
        assert_eq!(values.len(), 1);
        assert_eq!(values[0].to_str().unwrap(), format!("Bearer {token}"));
        assert!(values[0].is_sensitive());
    }

    #[test]
    fn test_build_client_request_preserves_path_and_query_without_token_in_uri() {
        let token = "test-token-A_2026";
        let request =
            build_client_request("ws://127.0.0.1:3001/onebot?scene=test&x=1", Some(token)).unwrap();
        assert_eq!(request.uri().path(), "/onebot");
        assert_eq!(request.uri().query(), Some("scene=test&x=1"));
        assert!(!request.uri().to_string().contains(token));
        let request_debug = format!("{request:?}");
        assert!(!request_debug.contains(token));
        let header_debug = format!("{:?}", request.headers().get(AUTHORIZATION).unwrap());
        assert!(!header_debug.contains(token));
    }

    #[test]
    fn test_build_client_request_rejects_empty_and_control_character_tokens() {
        for token in ["", "abc\rdef", "abc\ndef", "abc\r\ndef", "abc\0def"] {
            let error = build_client_request("ws://127.0.0.1:3001", Some(token)).unwrap_err();
            let display = format!("{error}");
            let debug = format!("{error:?}");
            assert!(display.contains("access_token"));
            if !token.is_empty() {
                assert!(!display.contains(token));
                assert!(!debug.contains(token));
            }
        }
    }

    #[test]
    fn test_build_client_request_observes_non_ascii_token_without_leaking() {
        let token = "test-token-令牌_2026";
        match build_client_request("ws://127.0.0.1:3001", Some(token)) {
            Ok(request) => {
                let value = request.headers().get(AUTHORIZATION).unwrap();
                assert!(value.is_sensitive());
                assert!(!format!("{value:?}").contains("令牌"));
            }
            Err(error) => {
                assert!(!format!("{error}").contains("令牌"));
                assert!(!format!("{error:?}").contains("令牌"));
            }
        }
    }

    #[test]
    fn test_build_client_request_rejects_invalid_urls_without_leaking_token() {
        for url in ["not a url", "ws:///onebot"] {
            let error = build_client_request(url, Some("test-token-A_2026")).unwrap_err();
            assert!(!format!("{error}").contains("test-token-A_2026"));
            assert!(!format!("{error:?}").contains("test-token-A_2026"));
        }
    }

    #[test]
    fn test_check_authorization_accepts_scheme_and_header_name_variants() {
        let expected = "test-token-A_2026";
        for scheme in ["Bearer", "bearer", "bEaReR"] {
            for header_name in ["Authorization", "authorization", "AUTHORIZATION"] {
                let mut headers = HeaderMap::new();
                headers.insert(
                    HeaderName::from_bytes(header_name.as_bytes()).unwrap(),
                    HeaderValue::from_str(&format!("{scheme} {expected}")).unwrap(),
                );
                assert_eq!(
                    check_authorization(&headers, expected),
                    Ok(()),
                    "scheme={scheme}, header={header_name}"
                );
            }
        }
    }

    #[test]
    fn test_check_authorization_malformed_categories() {
        let expected = "test-token-A_2026";
        let cases = [
            ("", AuthFailure::Malformed),
            ("Bearer", AuthFailure::Malformed),
            ("Bearer ", AuthFailure::Malformed),
            ("Bearer\ttest-token-A_2026", AuthFailure::Malformed),
            (" Bearer test-token-A_2026", AuthFailure::UnsupportedScheme),
            ("Bearer  test-token-A_2026", AuthFailure::InvalidToken),
            ("Bearer test-token-A_2026 ", AuthFailure::InvalidToken),
            ("Bearer test-token-A_2026 extra", AuthFailure::InvalidToken),
        ];
        for (value, expected_failure) in cases {
            let headers = authorization_headers(Some(value));
            assert_eq!(
                check_authorization(&headers, expected),
                Err(expected_failure),
                "value={value:?}"
            );
        }
    }

    #[test]
    fn test_check_authorization_rejects_other_schemes_even_with_correct_token() {
        let expected = "test-token-A_2026";
        for value in [format!("Basic {expected}"), format!("Digest {expected}")] {
            let headers = authorization_headers(Some(&value));
            assert_eq!(
                check_authorization(&headers, expected),
                Err(AuthFailure::UnsupportedScheme)
            );
        }
    }

    #[test]
    fn test_check_authorization_rejects_token_variants() {
        let expected = "test-token-A_2026";
        for value in [
            "Bearer test-token-B_2026",
            "Bearer TEST-TOKEN-A_2026",
            "Bearer test-token-A_202",
            "Bearer test-token-A_2026x",
            "Bearer test-token-A_202",
            "Bearer test-token-A%5F2026",
        ] {
            let headers = authorization_headers(Some(value));
            assert_eq!(
                check_authorization(&headers, expected),
                Err(AuthFailure::InvalidToken),
                "value={value:?}"
            );
        }
    }

    #[test]
    fn test_check_authorization_rejects_non_utf8_header_value() {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_bytes(b"Bearer \xFF").unwrap(),
        );
        assert_eq!(
            check_authorization(&headers, "test-token-A_2026"),
            Err(AuthFailure::Malformed)
        );
    }

    #[test]
    fn test_constant_time_eq_edge_cases() {
        assert!(constant_time_eq(b"", b""));
        assert!(!constant_time_eq(b"", b"a"));
        assert!(!constant_time_eq(b"a", b""));
        assert!(!constant_time_eq(b"abc", b"xbc"));
        assert!(!constant_time_eq(b"abc", b"axc"));
        assert!(!constant_time_eq(b"abc", b"abx"));
        assert!(!constant_time_eq(b"abc", b"abc\0"));
        assert!(!constant_time_eq(b"abc\0", b"abc"));
        assert!(constant_time_eq(b"\0", b"\0"));
    }

    #[test]
    fn test_unauthorized_response_has_complete_fixed_fields() {
        let response = unauthorized_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(response.body().as_deref(), Some("Unauthorized"));
        assert_eq!(
            response
                .headers()
                .get("WWW-Authenticate")
                .and_then(|value| value.to_str().ok()),
            Some("Bearer")
        );
        assert_eq!(
            response
                .headers()
                .get("Content-Type")
                .and_then(|value| value.to_str().ok()),
            Some("text/plain; charset=utf-8")
        );
        assert_eq!(
            response
                .headers()
                .get("Content-Length")
                .and_then(|value| value.to_str().ok()),
            Some("12")
        );
        assert_eq!(
            response
                .headers()
                .get("Connection")
                .and_then(|value| value.to_str().ok()),
            Some("close")
        );
    }

    fn handshake_request(headers: &[(&str, &str)]) -> HandshakeRequest {
        let mut request = HandshakeRequest::builder()
            .method("GET")
            .uri("/")
            .body(())
            .unwrap();
        for (name, value) in headers {
            request.headers_mut().insert(
                HeaderName::from_bytes(name.as_bytes()).unwrap(),
                HeaderValue::from_str(value).unwrap(),
            );
        }
        request
    }

    fn handshake_response() -> HandshakeResponse {
        HandshakeResponse::builder()
            .status(StatusCode::SWITCHING_PROTOCOLS)
            .body(())
            .unwrap()
    }

    #[test]
    fn test_authorize_handshake_without_configured_token_ignores_authorization() {
        let cases = [
            vec![],
            vec![("Authorization", "Bearer test-token-A_2026")],
            vec![("Authorization", "Bearer test-token-B_2026")],
            vec![("Authorization", "Basic anything")],
        ];
        for headers in cases {
            let request = handshake_request(&headers);
            let response = authorize_handshake(&request, handshake_response(), None).unwrap();
            assert_eq!(response.status(), StatusCode::SWITCHING_PROTOCOLS);
        }
    }

    async fn handshake_status(access_token: Option<&str>, authorization: Option<&str>) -> u16 {
        let (mut client, server) = tokio::io::duplex(4096);
        let access_token = access_token.map(str::to_owned);
        let server_task = tokio::spawn(async move { accept_websocket(server, access_token).await });

        let mut request = String::from(HANDSHAKE_REQUEST);
        if let Some(authorization) = authorization {
            request.push_str("Authorization: ");
            request.push_str(authorization);
            request.push_str("\r\n");
        }
        request.push_str("\r\n");
        client.write_all(request.as_bytes()).await.unwrap();

        let mut reader = BufReader::new(client);
        let mut status_line = String::new();
        reader.read_line(&mut status_line).await.unwrap();
        let status = status_line
            .split_whitespace()
            .nth(1)
            .and_then(|status| status.parse().ok())
            .expect("handshake response should contain a status code");

        // The handshake is complete; the server task should not wait for more connection data.
        let _ = server_task.await.unwrap();
        status
    }

    #[tokio::test]
    async fn test_handshake_without_configured_token_is_open() {
        assert_eq!(handshake_status(None, None).await, 101);
    }

    #[tokio::test]
    async fn test_handshake_accepts_valid_bearer_token() {
        assert_eq!(
            handshake_status(Some("secret-token"), Some("Bearer secret-token")).await,
            101
        );
    }

    #[tokio::test]
    async fn test_handshake_rejects_missing_token() {
        assert_eq!(handshake_status(Some("secret-token"), None).await, 401);
    }

    #[tokio::test]
    async fn test_handshake_rejects_wrong_token() {
        assert_eq!(
            handshake_status(Some("secret-token"), Some("Bearer wrong-token")).await,
            401
        );
    }

    #[tokio::test]
    async fn test_handshake_rejects_wrong_scheme_and_malformed_value() {
        for authorization in ["Basic secret-token", "Bearer", "Bearer "] {
            assert_eq!(
                handshake_status(Some("secret-token"), Some(authorization)).await,
                401
            );
        }
    }
}
