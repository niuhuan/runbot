//! Wraps the OneBot action `get_user_status`.
//!
//! The method sends a query by user QQ number, waits for the response with the
//! matching echo, checks `retcode`, and deserializes `data` into a
//! [`UserStatus`].

use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use tokio::time::Duration;

/// The online status of the specified user.
///
/// Both fields are required: a missing field or an explicit `null` causes
/// deserialization to fail with [`crate::error::Error::JsonError`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStatus {
    /// QQ number of the user in the result; this library does not check that it
    /// matches the request argument.
    pub user_id: i64,
    /// User status code; the values and meanings are defined by the upstream
    /// implementation. This library does not map it to an enum.
    pub status: i32,
}

impl BotContext {
    /// Queries the online status of the specified user and waits up to `timeout`
    /// for the response data.
    ///
    /// `timeout` limits only the response wait after the request has been
    /// written to the connection. It does not cover JSON serialization, internal
    /// lock acquisition, or the WebSocket write.
    ///
    /// # Arguments
    ///
    /// - `user_id`: QQ number of the user to query; this library does not check
    ///   whether it exists.
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
    /// be deserialized into [`UserStatus`].
    ///
    /// # Cancellation
    ///
    /// Dropping the pending future stops the local wait and removes the matching
    /// echo registration; the request has already been written to the
    /// connection, so cancellation does not withdraw it from the upstream.
    pub async fn get_user_status_with_timeout(
        &self,
        user_id: i64,
        timeout: Duration,
    ) -> Result<UserStatus> {
        let response = self
            .websocket_send(
                "get_user_status",
                json!({
                    "user_id": user_id,
                }),
            )
            .await?;
        let data = response.data(timeout).await?;
        let status: UserStatus = serde_json::from_value(data)?;
        Ok(status)
    }

    /// Queries the online status of the specified user using a 10-second
    /// response wait.
    ///
    /// Except for the fixed wait duration, the arguments, return value, and
    /// error semantics are the same as
    /// [`Self::get_user_status_with_timeout`].
    pub async fn get_user_status(&self, user_id: i64) -> Result<UserStatus> {
        self.get_user_status_with_timeout(user_id, Duration::from_secs(10))
            .await
    }
}

