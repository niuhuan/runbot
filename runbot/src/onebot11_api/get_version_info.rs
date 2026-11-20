//! Wraps the OneBot action `get_version_info`.
//!
//! The method sends a query request, waits for the response with the matching
//! echo, checks `retcode`, and deserializes `data` into a [`VersionInfo`].

use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use tokio::time::Duration;

/// Version information of the upstream OneBot implementation.
///
/// All three fields are required: a missing field or an explicit `null` causes
/// deserialization to fail with [`crate::error::Error::JsonError`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    /// Name of the upstream implementation, such as `NapCat` or `LLOneBot`.
    pub app_name: String,
    /// Version of the upstream implementation.
    pub app_version: String,
    /// OneBot protocol version declared by the upstream; this library does not
    /// validate compatibility against it.
    pub protocol_version: String,
}

impl BotContext {
    /// Requests version information from the upstream implementation and waits
    /// up to `timeout` for the response data.
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
    /// be deserialized into [`VersionInfo`].
    ///
    /// # Cancellation
    ///
    /// Dropping the pending future stops the local wait and removes the matching
    /// echo registration; the request has already been written to the
    /// connection, so cancellation does not withdraw it from the upstream.
    pub async fn get_version_info_with_timeout(&self, timeout: Duration) -> Result<VersionInfo> {
        let response = self
            .websocket_send("get_version_info", serde_json::Value::Null)
            .await?;
        let data = response.data(timeout).await?;
        let version: VersionInfo = serde_json::from_value(data)?;
        Ok(version)
    }

    /// Gets version information from the upstream implementation using a
    /// 10-second response wait.
    ///
    /// Except for the fixed wait duration, the arguments, return value, and
    /// error semantics are the same as
    /// [`Self::get_version_info_with_timeout`].
    pub async fn get_version_info(&self) -> Result<VersionInfo> {
        self.get_version_info_with_timeout(Duration::from_secs(10))
            .await
    }
}

