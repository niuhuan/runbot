//! Wraps the OneBot action `get_group_at_all_remain`.
//!
//! The method sends a query by group number, waits for the response with the
//! matching echo, checks `retcode`, and deserializes `data` into a
//! [`GroupAtAllRemain`].

use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use tokio::time::Duration;

/// Permission and remaining uses of @all in a group.
///
/// All three fields are required: a missing field or an explicit `null` causes
/// deserialization to fail with [`crate::error::Error::JsonError`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupAtAllRemain {
    /// Whether the current account can use @all in this group.
    pub can_at_all: bool,
    /// Remaining group-wide @all uses for this group.
    pub remain_at_all_count_for_group: i32,
    /// Remaining @all uses for the current account in this group.
    pub remain_at_all_count_for_uin: i32,
}

impl BotContext {
    /// Queries the remaining @all uses for the specified group and waits up to
    /// `timeout` for the response data.
    ///
    /// `timeout` limits only the response wait after the request has been
    /// written to the connection. It does not cover JSON serialization, internal
    /// lock acquisition, or the WebSocket write.
    ///
    /// # Arguments
    ///
    /// - `group_id`: number of the group to query.
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
    /// be deserialized into [`GroupAtAllRemain`].
    ///
    /// # Cancellation
    ///
    /// Dropping the pending future stops the local wait and removes the matching
    /// echo registration; the request has already been written to the
    /// connection, so cancellation does not withdraw it from the upstream.
    pub async fn get_group_at_all_remain_with_timeout(
        &self,
        group_id: i64,
        timeout: Duration,
    ) -> Result<GroupAtAllRemain> {
        let response = self
            .websocket_send(
                "get_group_at_all_remain",
                json!({
                    "group_id": group_id,
                }),
            )
            .await?;
        let data = response.data(timeout).await?;
        let remain: GroupAtAllRemain = serde_json::from_value(data)?;
        Ok(remain)
    }

    /// Queries the remaining @all uses for the specified group using a
    /// 10-second response wait.
    ///
    /// Except for the fixed wait duration, the arguments, return value, and
    /// error semantics are the same as
    /// [`Self::get_group_at_all_remain_with_timeout`].
    pub async fn get_group_at_all_remain(&self, group_id: i64) -> Result<GroupAtAllRemain> {
        self.get_group_at_all_remain_with_timeout(group_id, Duration::from_secs(10))
            .await
    }
}

