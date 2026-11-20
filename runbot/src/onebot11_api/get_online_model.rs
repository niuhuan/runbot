//! Wraps the OneBot action `get_online_model`.
//!
//! The method sends a query request, waits for the response with the matching
//! echo, checks `retcode`, and deserializes `data` into an [`OnlineModel`].

use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use tokio::time::Duration;

/// The online device model configured for the current account.
///
/// Both fields are required: a missing field or an explicit `null` causes
/// deserialization to fail with [`crate::error::Error::JsonError`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnlineModel {
    /// Device model identifier, such as a model code, supplied by the upstream.
    pub model: String,
    /// Device model name shown externally, supplied by the upstream.
    pub model_show: String,
}

impl BotContext {
    /// Requests the online device model configured for the current account and
    /// waits up to `timeout` for the response data.
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
    /// be deserialized into [`OnlineModel`].
    ///
    /// # Cancellation
    ///
    /// Dropping the pending future stops the local wait and removes the matching
    /// echo registration; the request has already been written to the
    /// connection, so cancellation does not withdraw it from the upstream.
    pub async fn get_online_model_with_timeout(&self, timeout: Duration) -> Result<OnlineModel> {
        let response = self
            .websocket_send("get_online_model", serde_json::Value::Null)
            .await?;
        let data = response.data(timeout).await?;
        let model: OnlineModel = serde_json::from_value(data)?;
        Ok(model)
    }

    /// Gets the online device model configured for the current account using a
    /// 10-second response wait.
    ///
    /// Except for the fixed wait duration, the arguments, return value, and
    /// error semantics are the same as
    /// [`Self::get_online_model_with_timeout`].
    pub async fn get_online_model(&self) -> Result<OnlineModel> {
        self.get_online_model_with_timeout(Duration::from_secs(10))
            .await
    }
}

