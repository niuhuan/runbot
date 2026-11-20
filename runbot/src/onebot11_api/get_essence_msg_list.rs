//! Gets the group essence message list (OneBot 11 action `get_essence_msg_list`).
//!
//! The default version waits up to ten seconds for a response; the `_with_timeout` version
//! allows a custom response wait duration.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use tokio::time::Duration;

/// An entry in the group essence message list.
///
/// All fields are required during deserialization; a missing field or a type mismatch results
/// in [`crate::error::Error::JsonError`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EssenceMsg {
    /// QQ ID of the original message sender.
    pub sender_id: i64,
    /// Nickname of the original message sender.
    pub sender_nick: String,
    /// Timestamp of the original message; the unit is determined by the upstream
    /// implementation (QQ-related implementations usually use Unix seconds).
    pub sender_time: i64,
    /// QQ ID of the operator who set the message as essence.
    pub operator_id: i64,
    /// Nickname of the operator.
    pub operator_nick: String,
    /// Timestamp when the message was set as essence; the unit matches
    /// [`Self::sender_time`].
    pub operator_time: i64,
    /// Message ID of the essence message.
    pub message_id: i64,
}

impl BotContext {
    /// Requests the essence message list of the specified group and waits for the response
    /// data within the specified time.
    ///
    /// An available connection should exist before calling this method. `timeout` limits the
    /// response wait after the request has been sent; it does not include serialization,
    /// lock acquisition, or the WebSocket write. This method does not retry and does not sort
    /// the returned list.
    ///
    /// # Arguments
    ///
    /// - `group_id`: ID of the group to query.
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
    ///   deserialized into a list of [`EssenceMsg`].
    ///
    /// # Cancellation
    ///
    /// Dropping the pending future stops the local wait and releases the response
    /// registration; the request already sent is not revoked from the upstream.
    pub async fn get_essence_msg_list_with_timeout(
        &self,
        group_id: i64,
        timeout: Duration,
    ) -> Result<Vec<EssenceMsg>> {
        let response = self
            .websocket_send(
                "get_essence_msg_list",
                json!({
                    "group_id": group_id,
                }),
            )
            .await?;
        let data = response.data(timeout).await?;
        let messages: Vec<EssenceMsg> = serde_json::from_value(data)?;
        Ok(messages)
    }

    /// Calls [`Self::get_essence_msg_list_with_timeout`] with a ten-second response wait.
    ///
    /// Connection, send, timeout, `retcode`, and deserialization errors are the same as in
    /// the version with a timeout; the list order is determined by the upstream response.
    pub async fn get_essence_msg_list(&self, group_id: i64) -> Result<Vec<EssenceMsg>> {
        self.get_essence_msg_list_with_timeout(group_id, Duration::from_secs(10))
            .await
    }
}

