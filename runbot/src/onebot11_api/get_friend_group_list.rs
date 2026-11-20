//! Gets the friend group list (upstream extension action `get_friend_group_list`).
//!
//! The default version waits up to ten seconds for a response; the `_with_timeout` version
//! allows a custom response wait duration.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use tokio::time::Duration;

/// An entry in the friend group list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriendGroup {
    /// Friend group ID; this is not a chat group ID. The numbering scheme is determined by
    /// the upstream implementation.
    pub group_id: i32,
    /// Group name.
    pub group_name: String,
    /// Number of friends in the group.
    pub friend_count: i32,
}

impl BotContext {
    /// Requests the friend group list and waits for the response data within the specified
    /// time.
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
    ///   deserialized into a list of [`FriendGroup`].
    ///
    /// # Cancellation
    ///
    /// Dropping the pending future stops the local wait and releases the response
    /// registration; the request already sent is not revoked from the upstream.
    pub async fn get_friend_group_list_with_timeout(
        &self,
        timeout: Duration,
    ) -> Result<Vec<FriendGroup>> {
        let response = self
            .websocket_send("get_friend_group_list", serde_json::Value::Null)
            .await?;
        let data = response.data(timeout).await?;
        let groups: Vec<FriendGroup> = serde_json::from_value(data)?;
        Ok(groups)
    }

    /// Calls [`Self::get_friend_group_list_with_timeout`] with a ten-second response wait.
    ///
    /// Connection, send, timeout, `retcode`, and deserialization errors are the same as in
    /// the version with a timeout; the list order is determined by the upstream response.
    pub async fn get_friend_group_list(&self) -> Result<Vec<FriendGroup>> {
        self.get_friend_group_list_with_timeout(Duration::from_secs(10))
            .await
    }
}

