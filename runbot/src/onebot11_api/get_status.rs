//! Wraps the OneBot action `get_status`.
//!
//! The method sends a query request, waits for the response with the matching
//! echo, checks `retcode`, and deserializes `data` into a [`Status`].

use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use tokio::time::Duration;

/// The current account's online status.
///
/// Both fields are required: a missing field or an explicit `null` causes
/// deserialization to fail with [`crate::error::Error::JsonError`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Status {
    /// Whether the account is online.
    pub online: bool,
    /// Whether the account status is healthy; the exact check is defined by the
    /// upstream implementation.
    pub good: bool,
}

impl BotContext {
    /// Requests the current account's online status and waits up to `timeout`
    /// for the response data.
    ///
    /// `timeout` limits only the response wait after the request has been
    /// written to the connection. It does not cover JSON serialization, internal
    /// lock acquisition, or the WebSocket write.
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
    /// be deserialized into [`Status`].
    ///
    /// # Cancellation
    ///
    /// Dropping the pending future stops the local wait and removes the matching
    /// echo registration; the request has already been written to the
    /// connection, so cancellation does not withdraw it from the upstream.
    pub async fn get_status_with_timeout(&self, timeout: Duration) -> Result<Status> {
        let response = self
            .websocket_send("get_status", serde_json::Value::Null)
            .await?;
        let data = response.data(timeout).await?;
        let status: Status = serde_json::from_value(data)?;
        Ok(status)
    }

    /// Gets the current account's online status using a 10-second response wait.
    ///
    /// Except for the fixed wait duration, the arguments, return value, and
    /// error semantics are the same as
    /// [`Self::get_status_with_timeout`].
    pub async fn get_status(&self) -> Result<Status> {
        self.get_status_with_timeout(Duration::from_secs(10))
            .await
    }
}

