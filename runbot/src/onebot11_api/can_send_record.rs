//! Checks whether the current account can send voice messages (OneBot 11 action
//! `can_send_record`).
//!
//! The method sends the request and waits up to ten seconds for a response.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use tokio::time::Duration;

/// Response data for `can_send_record`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanSendRecord {
    /// Whether the current account can send voice messages; the criteria are determined by
    /// the upstream implementation.
    pub yes: bool,
}

impl BotContext {
    /// Requests a check of whether the current account can send voice messages and waits up
    /// to ten seconds for the response data.
    ///
    /// A `false` value for [`CanSendRecord::yes`] means the upstream considers voice sending
    /// unavailable; it does not indicate a failed request. The criteria (for example,
    /// account permissions or risk-control state) are determined by the upstream
    /// implementation.
    ///
    /// # Errors
    ///
    /// - Returns [`crate::error::Error::StateError`] when no connection is available or the
    ///   response channel is closed.
    /// - Returns [`crate::error::Error::WebSocketError`] when the WebSocket send fails.
    /// - Returns [`crate::error::Error::TimeoutError`] when no response arrives within ten
    ///   seconds.
    /// - Returns [`crate::error::Error::StateError`] when the upstream response has a
    ///   non-zero `retcode`.
    /// - Returns [`crate::error::Error::JsonError`] when the response data cannot be
    ///   deserialized into [`CanSendRecord`].
    ///
    /// # Cancellation
    ///
    /// Dropping the pending future stops the local wait and releases the response
    /// registration; the request already sent is not revoked from the upstream.
    pub async fn can_send_record(&self) -> Result<CanSendRecord> {
        let response = self
            .websocket_send("can_send_record", serde_json::Value::Null)
            .await?;
        let response = response.data(Duration::from_secs(10)).await?;
        let data: CanSendRecord = serde_json::from_value(response)?;
        Ok(data)
    }
}
