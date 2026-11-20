//! Gets the like list (upstream extension action `get_like_list`).
//!
//! The default version waits up to ten seconds for a response; the `_with_timeout` version
//! allows a custom response wait duration.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use tokio::time::Duration;

/// An entry in the like list.
///
/// All fields are required during deserialization; a missing field or a type mismatch results
/// in [`crate::error::Error::JsonError`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LikeInfo {
    /// QQ ID of the user who liked.
    pub user_id: i64,
    /// Nickname of the user who liked.
    pub nickname: String,
    /// Timestamp of the like; the unit is determined by the upstream implementation
    /// (QQ-related implementations usually use Unix seconds).
    pub time: i64,
}

impl BotContext {
    /// Requests the like list and waits for the response data within the specified time.
    ///
    /// An available connection should exist before calling this method. `timeout` limits the
    /// response wait after the request has been sent; it does not include serialization,
    /// lock acquisition, or the WebSocket write. This method does not retry and does not sort
    /// the returned list.
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
    ///   deserialized into a list of [`LikeInfo`].
    ///
    /// # Cancellation
    ///
    /// Dropping the pending future stops the local wait and releases the response
    /// registration; the request already sent is not revoked from the upstream.
    pub async fn get_like_list_with_timeout(
        &self,
        timeout: Duration,
    ) -> Result<Vec<LikeInfo>> {
        let response = self
            .websocket_send("get_like_list", serde_json::Value::Null)
            .await?;
        let data = response.data(timeout).await?;
        let likes: Vec<LikeInfo> = serde_json::from_value(data)?;
        Ok(likes)
    }

    /// Calls [`Self::get_like_list_with_timeout`] with a ten-second response wait.
    ///
    /// Connection, send, timeout, `retcode`, and deserialization errors are the same as in
    /// the version with a timeout; the list order is determined by the upstream response.
    pub async fn get_like_list(&self) -> Result<Vec<LikeInfo>> {
        self.get_like_list_with_timeout(Duration::from_secs(10))
            .await
    }
}

