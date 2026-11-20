//! Wraps the OneBot action `get_online_clients`.
//!
//! The method sends a query request, waits for the response with the matching
//! echo, checks `retcode`, and deserializes `data` into a list of
//! [`OnlineClient`] values.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use tokio::time::Duration;

/// One online client of the current account.
///
/// All three fields are required: a missing field or an explicit `null` causes
/// deserialization to fail with [`crate::error::Error::JsonError`]; no default
/// values are used.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnlineClient {
    /// Client App ID, defined by the upstream implementation; this library does
    /// not convert it.
    pub app_id: i64,
    /// Login device name, such as a phone model or desktop client name.
    pub device_name: String,
    /// Device category label; the exact values are defined by the upstream
    /// implementation.
    pub device_kind: String,
}

impl BotContext {
    /// Requests the list of online clients for the current account and waits up
    /// to `timeout` for the response data.
    ///
    /// `timeout` limits only the response wait after the request has been
    /// written to the connection. It does not cover JSON serialization, internal
    /// lock acquisition, or the WebSocket write. The order of the returned list
    /// is determined by the upstream; this method does not sort it.
    ///
    /// # Arguments
    ///
    /// - `timeout`: how long to wait for the response with the matching echo.
    ///
    /// # Errors
    ///
    /// The wait follows the semantics of [`crate::bot_context::EchoAsyncResponse::data`]: returns
    /// [`crate::error::Error::StateError`] when the current connection is
    /// unavailable (not connected, disconnected, or closed) or the response
    /// channel closes early; [`crate::error::Error::WebSocketError`] when the
    /// WebSocket write fails; [`crate::error::Error::TimeoutError`] when no
    /// response arrives within `timeout`; and
    /// [`crate::error::Error::StateError`] when the upstream `retcode` is
    /// non-zero, with the error text taken from the response `message`. In
    /// addition, returns [`crate::error::Error::JsonError`] when `data` cannot
    /// be deserialized into `Vec<OnlineClient>`.
    ///
    /// # Cancellation
    ///
    /// Dropping the pending future stops the local wait and removes the matching
    /// echo registration; the request has already been written to the
    /// connection, so cancellation does not withdraw it from the upstream.
    pub async fn get_online_clients_with_timeout(
        &self,
        timeout: Duration,
    ) -> Result<Vec<OnlineClient>> {
        let response = self
            .websocket_send("get_online_clients", serde_json::Value::Null)
            .await?;
        let data = response.data(timeout).await?;
        let clients: Vec<OnlineClient> = serde_json::from_value(data)?;
        Ok(clients)
    }

    /// Gets the list of online clients for the current account using a 10-second
    /// response wait.
    ///
    /// Except for the fixed wait duration, the arguments, return value, and
    /// error semantics are the same as
    /// [`Self::get_online_clients_with_timeout`].
    pub async fn get_online_clients(&self) -> Result<Vec<OnlineClient>> {
        self.get_online_clients_with_timeout(Duration::from_secs(10))
            .await
    }
}

