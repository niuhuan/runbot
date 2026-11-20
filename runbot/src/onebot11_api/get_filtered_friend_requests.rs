//! Gets filtered friend requests (upstream extension action
//! `get_filtered_friend_requests`).
//!
//! The default version waits up to ten seconds for a response; the `_with_timeout` version
//! allows a custom response wait duration.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use tokio::time::Duration;

/// An entry in the filtered friend request list.
///
/// All fields are required during deserialization; a missing field or a type mismatch results
/// in [`crate::error::Error::JsonError`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilteredFriendRequest {
    /// Request ID used to process this friend request later; the exact way it is passed is
    /// determined by the upstream implementation.
    pub request_id: i64,
    /// Nickname of the requester.
    pub requester_nick: String,
    /// QQ ID of the requester.
    pub requester_id: i64,
    /// Verification message attached to the request.
    pub request_message: String,
    /// Timestamp of the request; the unit is determined by the upstream implementation
    /// (QQ-related implementations usually use Unix seconds).
    pub request_time: i64,
}

impl BotContext {
    /// Requests the filtered friend request list and waits for the response data within the
    /// specified time.
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
    ///   deserialized into a list of [`FilteredFriendRequest`].
    ///
    /// # Cancellation
    ///
    /// Dropping the pending future stops the local wait and releases the response
    /// registration; the request already sent is not revoked from the upstream.
    pub async fn get_filtered_friend_requests_with_timeout(
        &self,
        timeout: Duration,
    ) -> Result<Vec<FilteredFriendRequest>> {
        let response = self
            .websocket_send("get_filtered_friend_requests", serde_json::Value::Null)
            .await?;
        let data = response.data(timeout).await?;
        let requests: Vec<FilteredFriendRequest> = serde_json::from_value(data)?;
        Ok(requests)
    }

    /// Calls [`Self::get_filtered_friend_requests_with_timeout`] with a ten-second response
    /// wait.
    ///
    /// Connection, send, timeout, `retcode`, and deserialization errors are the same as in
    /// the version with a timeout; the list order is determined by the upstream response.
    pub async fn get_filtered_friend_requests(&self) -> Result<Vec<FilteredFriendRequest>> {
        self.get_filtered_friend_requests_with_timeout(Duration::from_secs(10))
            .await
    }
}

