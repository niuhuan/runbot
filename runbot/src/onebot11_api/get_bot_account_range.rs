//! Gets the bot account range (upstream extension action `get_bot_account_range`).
//!
//! The default version waits up to ten seconds for a response; the `_with_timeout` version
//! allows a custom response wait duration.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use tokio::time::Duration;

/// Bot account range returned by the upstream.
///
/// `min` and `max` are QQ ID numeric bounds; the exact purpose of the range and whether the
/// endpoints are inclusive are determined by the upstream implementation. All fields are
/// required during deserialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotAccountRange {
    /// Lower bound of the range (QQ ID).
    pub min: i64,
    /// Upper bound of the range (QQ ID).
    pub max: i64,
}

impl BotContext {
    /// Requests the bot account range and waits for the response data within the specified
    /// time.
    ///
    /// An available connection should exist before calling this method. `timeout` limits the
    /// response wait after the request has been sent; it does not include serialization,
    /// lock acquisition, or the WebSocket write. This method does not retry.
    ///
    /// # Errors
    ///
    /// - Returns [`crate::error::Error::StateError`] when no connection is available or the
    ///   response channel is closed.
    /// - Returns [`crate::error::Error::WebSocketError`] when the WebSocket send fails.
    /// - Returns [`crate::error::Error::TimeoutError`] when no response arrives within
    ///   `timeout`.
    /// - Returns [`crate::error::Error::StateError`] when the upstream response has a
    ///   non-zero `retcode`.
    /// - Returns [`crate::error::Error::JsonError`] when the response data cannot be
    ///   deserialized into [`BotAccountRange`].
    ///
    /// # Cancellation
    ///
    /// Dropping the pending future stops the local wait and releases the response
    /// registration; the request already sent is not revoked from the upstream.
    pub async fn get_bot_account_range_with_timeout(
        &self,
        timeout: Duration,
    ) -> Result<BotAccountRange> {
        let response = self
            .websocket_send("get_bot_account_range", serde_json::Value::Null)
            .await?;
        let data = response.data(timeout).await?;
        let range: BotAccountRange = serde_json::from_value(data)?;
        Ok(range)
    }

    /// Calls [`Self::get_bot_account_range_with_timeout`] with a ten-second response wait.
    ///
    /// Connection, send, timeout, `retcode`, and deserialization errors are the same as in
    /// the version with a timeout.
    pub async fn get_bot_account_range(&self) -> Result<BotAccountRange> {
        self.get_bot_account_range_with_timeout(Duration::from_secs(10))
            .await
    }
}

