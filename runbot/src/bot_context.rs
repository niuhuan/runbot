//! Bot connection context, builders, and asynchronous response handles.
//!
//! [`BotContext`] is built by [`BotContextBuilder`] and stores the processor list,
//! connection, and shutdown signal. [`BotServer`] and [`BotServerBuilder`] describe reverse
//! WebSocket server configuration. The connection itself is established by the loop
//! functions in [`crate::connection`].
//!
//! [`EchoAsyncResponse`] is the response handle for a sent request; a wait method consumes
//! the handle before the result can be obtained.

use std::fmt::{self, Debug};
use std::sync::Arc;
use std::vec;

use crate::error::{Error, Result};
use crate::event::*;
use crate::process::{Processor, loop_processors};
use async_trait::async_trait;
use dashmap::DashMap;
use futures_util::SinkExt;
use futures_util::stream::SplitSink;
use serde_json::json;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::Mutex;
use tokio::time::Duration;
use tokio_tungstenite::WebSocketStream;

/// Context shared by one bot connection. It stores the connection, processor list,
/// response-wait registry, and shutdown signal.
///
/// This type is usually created by [`BotContextBuilder::build`] and passed as
/// [`std::sync::Arc`] to the connection loop and each processor. Cloning the `Arc` does not
/// copy the connection or state; it shares the same context.
///
/// In forward mode, [`crate::connection::loop_client`] fills the internal connection field
/// after connecting. In reverse mode, [`crate::connection::loop_server`] creates a separate
/// context for each connection that completes the handshake. While no internal connection is
/// ready, methods that send requests return [`Error::StateError`].
///
/// The `Debug` implementation renders a configured access token as `[REDACTED]`.
// todo: default time for bot context
pub struct BotContext {
    pub(crate) connection: Mutex<Option<BotConnection>>,
    /// Forward WebSocket upstream address; `None` for contexts created by reverse
    /// connections.
    ///
    /// [`crate::connection::loop_client`] reads and copies this address at startup, so
    /// changing the field later does not retarget a running connection loop. Rebuild the
    /// context and restart the loop to change the address.
    pub url: Option<String>,
    /// Reserved bot identifier field.
    ///
    /// The library currently initializes this field to `0` and never updates it from the
    /// `self_id` reported by OneBot. Read `self_id` from events instead of relying on this
    /// field.
    pub id: i64,
    /// Processor list in registration order.
    ///
    /// For each event, processors are called from first to last until one returns
    /// `Ok(true)` or an error. The list is shared through [`std::sync::Arc`] after
    /// construction and cannot be mutated through this field; rebuild the context to add or
    /// remove processors.
    pub processors: Arc<Vec<Processor>>,
    /// Registry of echo identifiers waiting for API responses.
    ///
    /// [`BotContext::websocket_send`] generates an echo and registers a sender for each
    /// request. When a matching response arrives, the internal receive logic removes the
    /// entry and delivers the response. The field is public for internal access; modifying
    /// it directly breaks response waiting and is not part of the stable API.
    pub echo_notifer: Arc<DashMap<String, tokio::sync::mpsc::Sender<Response>>>,
    pub(crate) shutdown_tx: Mutex<Option<tokio::sync::watch::Sender<bool>>>,
    pub(crate) shutdown_rx: Mutex<Option<tokio::sync::watch::Receiver<bool>>>,
    pub(crate) access_token: Option<String>,
}

impl Debug for BotContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BotContext")
            .field("connection", &self.connection)
            .field("url", &self.url)
            .field("id", &self.id)
            .field("processors", &self.processors)
            .field("echo_notifer", &self.echo_notifer)
            .field("shutdown_tx", &self.shutdown_tx)
            .field("shutdown_rx", &self.shutdown_rx)
            .field(
                "access_token",
                &self.access_token.as_ref().map(|_| "[REDACTED]"),
            )
            .finish()
    }
}

/// Handle for waiting on the OneBot API response of a sent request.
///
/// Created by [`BotContext::websocket_send`] and by OneBot methods that return an
/// asynchronous response. It only means the request was written to the current WebSocket;
/// it does not mean the remote side has processed it. Call [`Self::response`],
/// [`Self::data`], or [`Self::ok`] to wait for the result.
///
/// All three wait methods consume `self`, so each handle can be awaited only once. Dropping
/// the handle removes its echo from the response registry but does not cancel the request
/// already sent to the remote side. A timeout or cancelled wait likewise does not prove the
/// remote side skipped execution.
#[derive(Debug)]
pub struct EchoAsyncResponse(
    String,
    tokio::sync::mpsc::Receiver<Response>,
    Arc<DashMap<String, tokio::sync::mpsc::Sender<Response>>>,
);

impl Drop for EchoAsyncResponse {
    fn drop(&mut self) {
        self.2.remove(&self.0);
    }
}

impl EchoAsyncResponse {
    /// Waits up to `timeout` for the raw API response matching this handle's echo.
    ///
    /// The returned [`Response`] preserves the upstream `retcode`, `message`, `wording`,
    /// and `data`. This method does not check the business status code; a non-zero
    /// `retcode` is still returned to the caller as a successful response.
    ///
    /// # Arguments
    ///
    /// - `timeout`: response wait after the request has been sent. It does not include
    ///   serialization, lock acquisition, or network write time.
    ///
    /// # Errors
    ///
    /// - Returns [`Error::TimeoutError`] when no response arrives within `timeout`.
    /// - Returns [`Error::StateError`] when the response channel closes before a message
    ///   arrives.
    ///
    /// # Cancellation
    ///
    /// Dropping the waiting future stops the local wait; the request already sent to the
    /// remote side is not cancelled.
    pub async fn response(mut self, timeout: Duration) -> Result<Response> {
        let r = tokio::time::timeout(timeout, async { self.1.recv().await }).await;
        let r = match r {
            Ok(r) => r,
            Err(err) => {
                return Err(Error::TimeoutError(
                    format!("response timeout for wait echo : {}", self.0),
                    err,
                ));
            }
        };
        match r {
            Some(r) => Ok(r),
            None => Err(Error::StateError(format!(
                "response not received : {}",
                self.0
            ))),
        }
    }

    /// Waits up to `timeout` for the response and returns its `data` field.
    ///
    /// Unlike [`Self::response`], this method checks `retcode`: a non-zero value returns
    /// [`Error::StateError`] with the response's `message` as the error text. On success it
    /// returns the raw JSON value and does not deserialize it into an API-specific type.
    ///
    /// # Errors
    ///
    /// Inherits the timeout and closed-channel errors of [`Self::response`]; returns
    /// [`Error::StateError`] when the upstream `retcode` is non-zero.
    pub async fn data(self, timeout: Duration) -> Result<serde_json::Value> {
        let r = self.response(timeout).await?;
        if r.retcode != 0 {
            return Err(Error::StateError(r.message));
        } else {
            Ok(r.data)
        }
    }

    /// Waits up to `timeout` for the response and checks only the OneBot business status.
    ///
    /// Returns `Ok(())` when `retcode` is zero; otherwise returns [`Error::StateError`] with
    /// the response's `message` as the error text. This method does not deserialize `data`
    /// and is suitable for operations whose response data is not needed.
    ///
    /// # Errors
    ///
    /// Inherits the timeout and closed-channel errors of [`Self::response`]; returns
    /// [`Error::StateError`] when the upstream `retcode` is non-zero.
    pub async fn ok(self, timeout: Duration) -> Result<()> {
        let r = self.response(timeout).await?;
        if r.retcode != 0 {
            return Err(Error::StateError(r.message));
        } else {
            Ok(())
        }
    }
}

impl BotContext {
    /// Sends a OneBot API request and returns a handle for the matching response.
    ///
    /// The request is encoded as `{"action": ..., "params": ..., "echo": ...}`. This method
    /// generates the `echo` and registers it in `echo_notifer`. Returning `Ok` means the
    /// request was written to the current WebSocket; it does not mean the upstream accepted
    /// or executed it. To obtain a result, call [`EchoAsyncResponse::response`],
    /// [`EchoAsyncResponse::data`], or [`EchoAsyncResponse::ok`], or use a higher-level
    /// OneBot method.
    ///
    /// # Arguments
    ///
    /// - `action`: OneBot action name, such as `send_private_msg`.
    /// - `msg`: JSON value used as the action's `params`; pass
    ///   [`serde_json::Value::Null`] when there are no parameters.
    ///
    /// # Errors
    ///
    /// Returns [`Error::StateError`] when the context has no usable connection (not
    /// connected, disconnected, or shut down); returns [`Error::WebSocketError`] when the
    /// WebSocket write fails. Upstream business errors are not returned here; they appear
    /// when waiting for the response.
    ///
    /// # Cancellation
    ///
    /// Dropping the returned handle removes the local echo registration but does not cancel
    /// the request already sent.
    pub async fn websocket_send(
        &self,
        action: &str,
        msg: serde_json::Value,
    ) -> Result<EchoAsyncResponse> {
        let echo = uuid::Uuid::new_v4().to_string();
        let (sender, receiver) = tokio::sync::mpsc::channel::<Response>(1);
        self.echo_notifer.insert(echo.clone(), sender);
        let echo_response = EchoAsyncResponse(echo.clone(), receiver, self.echo_notifer.clone());
        let msg = json!(
            {
                "action": action,
                "params": msg,
                "echo": echo
            }
        );
        let msg = serde_json::to_string(&msg).unwrap();
        tracing::debug!("WS send: {}", msg);
        let mut connection_lock = self.connection.lock().await;
        let connection = connection_lock
            .as_mut()
            .ok_or(Error::StateError("connection not ready".to_string()))?;
        connection.send_raw(msg).await?;
        Ok(echo_response)
    }
}

impl BotContext {
    pub(crate) async fn set_connection(&self, connection: impl Into<Option<BotConnection>>) {
        let mut connection_lock = self.connection.lock().await;
        *connection_lock = connection.into();
    }

    pub(crate) async fn handle_receive(
        &self,
        bot_ctx: Arc<BotContext>,
        msg: &tokio_tungstenite::tungstenite::protocol::Message,
    ) {
        match msg {
            tokio_tungstenite::tungstenite::protocol::Message::Text(text) => {
                tracing::debug!("WS received: {}", text.to_string());
                match parse_post(text) {
                    Ok(post) => {
                        tracing::debug!("parse post: {:?}", post);
                        if let Post::Response(response) = &post {
                            if let Some(v) = bot_ctx.echo_notifer.remove(&response.echo) {
                                match v.1.send(response.clone()).await {
                                    Ok(_) => {}
                                    Err(err) => {
                                        tracing::warn!("echo map error : {:?}", err);
                                    }
                                }
                            }
                        }
                        let _ = loop_processors(bot_ctx, self.processors.iter(), &post).await;
                    }
                    Err(e) => {
                        tracing::error!("Parse post error: {:?}", e);
                    }
                }
            }
            _ => {
                tracing::error!("WS received: {:?}", msg);
            }
        }
    }

    /// Returns a shared reference to the processor list registered in this context.
    ///
    /// The returned [`std::sync::Arc`] points to the same list held by the context and does
    /// not copy processors. List order is the event dispatch order. This method does not
    /// provide mutation; modules or menus can read the list to display available features.
    pub fn processors(&self) -> Arc<Vec<Processor>> {
        self.processors.clone()
    }

    /// Checks whether the context has entered the shutdown state.
    ///
    /// Returns `true` after [`Self::shutdown`] clears the shutdown signal channel, or when
    /// the signal has already been set to `true`. The implementation uses a non-blocking
    /// lock and returns `false` when another task holds it, so treat this as a hint in loops
    /// rather than a strict synchronization primitive.
    pub fn is_shutdown(&self) -> bool {
        // If shutdown_rx is None, shutdown has already completed.
        let shutdown_rx_guard = self.shutdown_rx.try_lock();
        if let Ok(shutdown_rx) = shutdown_rx_guard {
            if shutdown_rx.is_none() {
                return true;
            }
            if let Some(receiver) = shutdown_rx.as_ref() {
                return *receiver.borrow();
            }
        }
        false
    }

    /// Signals shutdown and disconnects the local connection.
    ///
    /// The first call sends `true` through the shutdown channel. After a successful send it
    /// clears the sender and receiver and empties the internal connection; repeated calls
    /// still return success. A running [`crate::connection::loop_client`] observes the
    /// signal at its next check, connection wait, or reconnect wait, and returns.
    ///
    /// This method performs local shutdown only. It does not wait for tasks spawned by
    /// processors, and it does not recall requests already sent to the remote side. The
    /// lifetime of each reverse-server connection is managed by
    /// [`crate::connection::loop_server`]; this method does not provide an overall server
    /// shutdown operation.
    ///
    /// # Errors
    ///
    /// Returns [`Error::StateError`] when the shutdown receiver has been dropped and the
    /// signal cannot be sent.
    pub async fn shutdown(&self) -> Result<()> {
        let mut shutdown_tx = self.shutdown_tx.lock().await;
        if let Some(ref sender) = *shutdown_tx {
            sender.send(true).map_err(|_| {
                Error::StateError("Failed to send shutdown signal".to_string())
            })?;
            // Clear the sender and receiver and mark the context as shut down.
            *shutdown_tx = None;
            let mut shutdown_rx = self.shutdown_rx.lock().await;
            *shutdown_rx = None;
        }
        // Disconnect.
        self.set_connection(None).await;
        Ok(())
    }
}

/// Abstraction for writing text frames to a WebSocket connection.
///
/// Implementors send `msg` as a single text frame. `Ok(())` only means the frame was
/// written to the underlying connection; it does not mean the peer has read or processed the
/// message. runbot provides a generic implementation for
/// `SplitSink<WebSocketStream<S>, Message>`; `BotContext` holds its sender through this
/// trait.
#[async_trait]
pub trait WsWriter {
    /// Sends `msg` as one text frame.
    ///
    /// # Errors
    ///
    /// Returns [`Error::WebSocketError`] when the underlying WebSocket is closed or the
    /// write fails.
    async fn send_raw(&mut self, msg: String) -> Result<()>;
}

// Generic implementation.
#[async_trait]
impl<S> WsWriter
    for SplitSink<WebSocketStream<S>, tokio_tungstenite::tungstenite::protocol::Message>
where
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    async fn send_raw(&mut self, msg: String) -> Result<()> {
        self.send(tokio_tungstenite::tungstenite::protocol::Message::Text(
            msg.into(),
        ))
        .await?;
        Ok(())
    }
}

pub(crate) struct BotConnection {
    pub(crate) sender: Box<dyn WsWriter + Send + Sync>,
}

impl BotConnection {
    pub async fn send_raw(&mut self, msg: String) -> Result<()> {
        self.sender.send_raw(msg).await?;
        Ok(())
    }
}

impl fmt::Debug for BotConnection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BotConnection")
            .field("sender", &"Box<dyn WsWriter>")
            .finish()
    }
}
/// Configuration builder for the forward WebSocket client.
///
/// Use chained calls to set the upstream address, access token, and processors, then call
/// [`Self::build`] to create a shared [`BotContext`]. Building does not open a network
/// connection; [`crate::connection::loop_client`] establishes it.
pub struct BotContextBuilder {
    /// Forward WebSocket upstream address.
    ///
    /// It can be modified directly before building. [`Self::build`] does not require this
    /// field; when the address is missing, [`crate::connection::loop_client`] returns
    /// [`Error::ParamsError`].
    pub url: Option<String>,
    /// Processors added at build time, in order.
    ///
    /// It can be modified directly, but [`Self::add_processor`] is usually preferred to keep
    /// the chained builder and conversion semantics.
    pub processors: Vec<Processor>,
    access_token: Option<String>,
}

impl BotContextBuilder {
    /// Creates a builder with no address, no credentials, and no processors.
    pub fn new() -> Self {
        Self {
            url: None,
            processors: vec![],
            access_token: None,
        }
    }

    /// Sets the forward WebSocket upstream address, such as `ws://127.0.0.1:3001`.
    ///
    /// Repeated calls replace the previous value. This method only stores the string and
    /// does not validate URL syntax; an invalid address causes an error when
    /// [`crate::connection::loop_client`] builds the connection request.
    pub fn url(self, url: impl Into<String>) -> Self {
        Self {
            url: Some(url.into()),
            ..self
        }
    }

    /// Sets the Bearer credential that the forward WebSocket client sends during the
    /// handshake.
    ///
    /// When this method is not called, no `Authorization` header is added. Once configured,
    /// both the initial connection and reconnects send `Authorization: Bearer <token>`.
    /// Repeated calls replace the previous value.
    ///
    /// This method only stores configuration. An empty string is rejected by [`Self::build`];
    /// other characters that cannot form an HTTP header cause an error when
    /// [`crate::connection::loop_client`] builds the connection request. The token is never
    /// added to the URL and is not written to `Debug` output.
    ///
    /// Bearer authentication does not encrypt the transport; protect credential transport at
    /// the deployment layer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use runbot::bot_context::BotContextBuilder;
    ///
    /// # fn main() -> runbot::error::Result<()> {
    /// let bot = BotContextBuilder::new()
    ///     .url("ws://127.0.0.1:3001")
    ///     .access_token("example-token")
    ///     .build()?;
    /// assert_eq!(bot.url.as_deref(), Some("ws://127.0.0.1:3001"));
    /// # Ok(())
    /// # }
    /// ```
    pub fn access_token(mut self, token: impl Into<String>) -> Self {
        self.access_token = Some(token.into());
        self
    }

    /// Appends an event processor.
    ///
    /// `processor` can be a value implementing a processor trait, a boxed trait object, or a
    /// static instance generated by `#[processor]`. Processors are stored in call order and
    /// run in sequence for each event until one returns `Ok(true)` or an error.
    ///
    /// Duplicate calls are not deduplicated; the same processor can be registered more than
    /// once.
    pub fn add_processor(
        mut self,
        processor: impl Into<Processor> + Sync + Send + 'static,
    ) -> Self {
        self.processors.push(processor.into());
        self
    }

    /// Builds a shared [`BotContext`] without opening a network connection.
    ///
    /// The returned context is not connected; API methods that send requests return
    /// [`Error::StateError`] until [`crate::connection::loop_client`] establishes a
    /// connection. The builder does not require [`Self::url`] to be set.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ParamsError`] when the access token is configured as an empty
    /// string.
    pub fn build(self) -> Result<Arc<BotContext>> {
        if self.access_token.as_deref() == Some("") {
            return Err(Error::ParamsError(
                "access_token must not be empty".to_string(),
            ));
        }
        let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
        Ok(Arc::new(BotContext {
            connection: Mutex::new(None),
            url: self.url,
            id: 0,
            processors: Arc::new(self.processors),
            echo_notifer: Arc::new(DashMap::new()),
            shutdown_tx: Mutex::new(Some(shutdown_tx)),
            shutdown_rx: Mutex::new(Some(shutdown_rx)),
            access_token: self.access_token,
        }))
    }
}

/// Reverse WebSocket server configuration built by [`BotServerBuilder`].
///
/// [`crate::connection::loop_server`] binds the address and creates an independent
/// [`BotContext`] for each inbound connection. This struct only stores configuration and
/// does not own a listener. When an access token is configured, the handshake validates
/// `Authorization: Bearer`; failures return 401 and do not create a context.
///
/// The `Debug` implementation renders the access token as `[REDACTED]`.
pub struct BotServer {
    /// Server listen address, such as `0.0.0.0:3131`.
    ///
    /// Building only checks that the value is present; it does not validate the format or
    /// port availability. The actual bind happens in
    /// [`crate::connection::loop_server`].
    pub bind: String,
    /// Processor list shared by all inbound connections, dispatched in registration order.
    pub processors: Arc<Vec<Processor>>,
    pub(crate) access_token: Option<String>,
}

impl Debug for BotServer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BotServer")
            .field("bind", &self.bind)
            .field("processors", &self.processors)
            .field(
                "access_token",
                &self.access_token.as_ref().map(|_| "[REDACTED]"),
            )
            .finish()
    }
}

/// Configuration builder for the reverse WebSocket server.
///
/// Use chained calls to set the listen address, access token, and processors, then call
/// [`Self::build`] to create a shared [`BotServer`]. Building does not bind a port;
/// [`crate::connection::loop_server`] performs the bind and listen.
pub struct BotServerBuilder {
    /// Reverse WebSocket listen address.
    ///
    /// It can be modified directly before building. [`Self::build`] requires this field to
    /// be present but does not validate the address format.
    pub bind: Option<String>,
    /// Processors added at build time, in order.
    pub processors: Vec<Processor>,
    access_token: Option<String>,
}

impl BotServerBuilder {
    /// Creates a builder with no listen address, no credentials, and no processors.
    pub fn new() -> Self {
        Self {
            bind: None,
            processors: vec![],
            access_token: None,
        }
    }

    /// Sets the reverse WebSocket listen address, such as `0.0.0.0:3131`.
    ///
    /// Repeated calls replace the previous value. This method only stores the string and
    /// does not parse it; an invalid format or an occupied port is returned as
    /// [`Error::StateError`] during the bind phase in
    /// [`crate::connection::loop_server`].
    pub fn bind(mut self, bind: impl Into<String>) -> Self {
        self.bind = Some(bind.into());
        self
    }

    /// Sets the Bearer credential required by the reverse WebSocket handshake.
    ///
    /// When this method is not called, the `Authorization` header is not validated and
    /// connections remain unauthenticated. Once configured, every inbound connection must
    /// carry a matching `Authorization: Bearer <token>` in the handshake request. A missing,
    /// malformed, unsupported, or mismatched credential receives 401 and no connection
    /// context is created. Repeated calls replace the previous value.
    ///
    /// This method only stores configuration. An empty string is rejected by
    /// [`Self::build`]. The token is never written to logs or `Debug` output. Bearer
    /// authentication does not encrypt the transport; protect credential transport at the
    /// deployment layer.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use runbot::bot_context::BotServerBuilder;
    ///
    /// # fn main() -> runbot::error::Result<()> {
    /// let server = BotServerBuilder::new()
    ///     .bind("127.0.0.1:3131")
    ///     .access_token("example-token")
    ///     .build()?;
    /// assert_eq!(server.bind, "127.0.0.1:3131");
    /// # Ok(())
    /// # }
    /// ```
    pub fn access_token(mut self, token: impl Into<String>) -> Self {
        self.access_token = Some(token.into());
        self
    }

    /// Appends an event processor shared by all inbound connections.
    ///
    /// Processors are stored in call order and run in sequence for each event until one
    /// returns `Ok(true)` or an error. Duplicate calls are not deduplicated.
    pub fn add_processor(
        mut self,
        processor: impl Into<Processor> + Sync + Send + 'static,
    ) -> Self {
        self.processors.push(processor.into());
        self
    }

    /// Builds a shared [`BotServer`] without binding a port.
    ///
    /// Successful construction only means the configuration fields are present; it does not
    /// mean the listen address is available. Binding and listening happen in
    /// [`crate::connection::loop_server`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::ParamsError`] when [`Self::bind`] is not set or the access token is
    /// configured as an empty string.
    pub fn build(self) -> Result<Arc<BotServer>> {
        if self.access_token.as_deref() == Some("") {
            return Err(Error::ParamsError(
                "access_token must not be empty".to_string(),
            ));
        }
        Ok(Arc::new(BotServer {
            bind: if let Some(bind) = self.bind {
                bind
            } else {
                return Err(Error::ParamsError("bind must be set".to_string()));
            },
            processors: Arc::new(self.processors),
            access_token: self.access_token,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_builder_stores_access_token() {
        let server = BotServerBuilder::new()
            .bind("127.0.0.1:3131")
            .access_token("secret-token")
            .build()
            .unwrap();
        assert_eq!(server.access_token.as_deref(), Some("secret-token"));
    }

    #[test]
    fn test_server_builder_rejects_empty_access_token() {
        let result = BotServerBuilder::new()
            .bind("127.0.0.1:3131")
            .access_token("")
            .build();
        assert!(
            matches!(result, Err(Error::ParamsError(message)) if message.contains("access_token"))
        );
    }

    #[test]
    fn test_server_debug_redacts_access_token() {
        let server = BotServerBuilder::new()
            .bind("127.0.0.1:3131")
            .access_token("secret-token")
            .build()
            .unwrap();
        let output = format!("{server:?}");
        assert!(!output.contains("secret-token"));
        assert!(output.contains("[REDACTED]"));
    }

    #[test]
    fn test_context_builder_stores_access_token() {
        let bot_ctx = BotContextBuilder::new()
            .url("ws://127.0.0.1:3131")
            .access_token("secret-token")
            .build()
            .unwrap();
        assert_eq!(bot_ctx.access_token.as_deref(), Some("secret-token"));
    }

    #[test]
    fn test_context_builder_rejects_empty_access_token() {
        let result = BotContextBuilder::new()
            .url("ws://127.0.0.1:3131")
            .access_token("")
            .build();
        assert!(
            matches!(result, Err(Error::ParamsError(message)) if message.contains("access_token"))
        );
    }

    #[test]
    fn test_context_debug_redacts_access_token() {
        let bot_ctx = BotContextBuilder::new()
            .url("ws://127.0.0.1:3131")
            .access_token("secret-token")
            .build()
            .unwrap();
        let output = format!("{bot_ctx:?}");
        assert!(!output.contains("secret-token"));
        assert!(output.contains("[REDACTED]"));
    }

    #[test]
    fn test_builders_default_to_no_access_token() {
        let server = BotServerBuilder::new()
            .bind("127.0.0.1:3131")
            .build()
            .unwrap();
        assert!(server.access_token.is_none());

        let bot_ctx = BotContextBuilder::new()
            .url("ws://127.0.0.1:3131")
            .build()
            .unwrap();
        assert!(bot_ctx.access_token.is_none());
    }

    #[test]
    fn test_builders_store_owned_and_borrowed_tokens_exactly() {
        let owned = String::from("Case-Sensitive_Token-2026");
        let server = BotServerBuilder::new()
            .bind("127.0.0.1:3131")
            .access_token(owned.clone())
            .build()
            .unwrap();
        assert_eq!(server.access_token.as_deref(), Some(owned.as_str()));

        let bot_ctx = BotContextBuilder::new()
            .url("ws://127.0.0.1:3131")
            .access_token("  token with spaces  ")
            .build()
            .unwrap();
        assert_eq!(
            bot_ctx.access_token.as_deref(),
            Some("  token with spaces  ")
        );
    }

    #[test]
    fn test_builders_use_last_configured_token() {
        let server = BotServerBuilder::new()
            .bind("127.0.0.1:3131")
            .access_token("token-A")
            .access_token("token-B")
            .build()
            .unwrap();
        assert_eq!(server.access_token.as_deref(), Some("token-B"));

        let bot_ctx = BotContextBuilder::new()
            .url("ws://127.0.0.1:3131")
            .access_token(String::from("token-A"))
            .access_token("token-B")
            .build()
            .unwrap();
        assert_eq!(bot_ctx.access_token.as_deref(), Some("token-B"));
    }

    #[test]
    fn test_server_builder_requires_bind() {
        let result = BotServerBuilder::new().access_token("token-A").build();
        assert!(matches!(result, Err(Error::ParamsError(message)) if message.contains("bind")));
    }

    #[tokio::test]
    async fn test_client_build_without_url_then_loop_client_returns_params_error() {
        let bot_ctx = BotContextBuilder::new()
            .access_token("token-A")
            .build()
            .unwrap();
        assert!(bot_ctx.url.is_none());

        let result = crate::connection::loop_client(bot_ctx).await;
        assert!(matches!(result, Err(Error::ParamsError(message)) if message.contains("url")));
    }

    #[test]
    fn test_debug_without_token_does_not_fake_credentials() {
        let server = BotServerBuilder::new()
            .bind("127.0.0.1:3131")
            .build()
            .unwrap();
        let server_debug = format!("{server:?}");
        assert!(server_debug.contains("127.0.0.1:3131"));
        assert!(!server_debug.contains("[REDACTED]"));
        assert!(!server_debug.contains("Bearer"));

        let bot_ctx = BotContextBuilder::new()
            .url("ws://127.0.0.1:3131")
            .build()
            .unwrap();
        let context_debug = format!("{bot_ctx:?}");
        assert!(context_debug.contains("ws://127.0.0.1:3131"));
        assert!(!context_debug.contains("[REDACTED]"));
        assert!(!context_debug.contains("Bearer"));
    }

    #[test]
    fn test_expanded_debug_redacts_tokens() {
        let server = BotServerBuilder::new()
            .bind("127.0.0.1:3131")
            .access_token("server-secret-token")
            .build()
            .unwrap();
        let server_debug = format!("{server:#?}");
        assert!(!server_debug.contains("server-secret-token"));
        assert!(server_debug.contains("[REDACTED]"));

        let bot_ctx = BotContextBuilder::new()
            .url("ws://127.0.0.1:3131")
            .access_token("client-secret-token")
            .build()
            .unwrap();
        let context_debug = format!("{bot_ctx:#?}");
        assert!(!context_debug.contains("client-secret-token"));
        assert!(context_debug.contains("[REDACTED]"));
    }
}
