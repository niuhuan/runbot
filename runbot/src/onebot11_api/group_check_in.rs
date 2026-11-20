//! Wraps the OneBot action `group_check_in` (group check-in).
//!
//! The method sends a check-in request by group number, waits for the response
//! with the matching echo, checks `retcode`, and deserializes `data` into a
//! [`GroupCheckIn`].

use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use tokio::time::Duration;

/// Result of a group check-in.
///
/// Both fields are required: a missing field or an explicit `null` causes
/// deserialization to fail with [`crate::error::Error::JsonError`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupCheckIn {
    /// Check-in result code; the values and meanings are defined by the upstream
    /// implementation. This library checks only the outer response `retcode`.
    pub result: i32,
    /// Points earned or the point change from the check-in; the unit and meaning
    /// are defined by the upstream implementation.
    pub point: i32,
}

impl BotContext {
    /// Performs a check-in for the specified group and waits up to `timeout` for
    /// the response data.
    ///
    /// `timeout` limits only the response wait after the request has been
    /// written to the connection. It does not cover JSON serialization, internal
    /// lock acquisition, or the WebSocket write. Group membership, daily limits,
    /// and other restrictions are determined by the upstream.
    ///
    /// # Arguments
    ///
    /// - `group_id`: number of the group to check in.
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
    /// be deserialized into [`GroupCheckIn`].
    ///
    /// # Cancellation
    ///
    /// Dropping the pending future stops the local wait and removes the matching
    /// echo registration; the request has already been written to the
    /// connection, so cancellation does not withdraw it from the upstream.
    pub async fn group_check_in_with_timeout(
        &self,
        group_id: i64,
        timeout: Duration,
    ) -> Result<GroupCheckIn> {
        let response = self
            .websocket_send(
                "group_check_in",
                json!({
                    "group_id": group_id,
                }),
            )
            .await?;
        let data = response.data(timeout).await?;
        let check_in: GroupCheckIn = serde_json::from_value(data)?;
        Ok(check_in)
    }

    /// Performs a check-in for the specified group using a 10-second response
    /// wait.
    ///
    /// Except for the fixed wait duration, the arguments, return value, and
    /// error semantics are the same as
    /// [`Self::group_check_in_with_timeout`].
    pub async fn group_check_in(&self, group_id: i64) -> Result<GroupCheckIn> {
        self.group_check_in_with_timeout(group_id, Duration::from_secs(10))
            .await
    }
}

