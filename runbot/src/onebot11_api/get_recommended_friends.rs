//! Wraps the OneBot action `get_recommended_friends`.
//!
//! The method sends a query request, waits for the response with the matching
//! echo, checks `retcode`, and deserializes `data` into a list of
//! [`RecommendedFriend`] values, usually from recommended-friend cards.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use tokio::time::Duration;

/// One friend from a recommended-friend card.
///
/// All seven fields are required: a missing field or an explicit `null` causes
/// deserialization to fail with [`crate::error::Error::JsonError`]. Field values are
/// supplied by the upstream; this library does not validate them.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendedFriend {
    /// QQ number of the recommended friend.
    pub user_id: i64,
    /// Nickname of the recommended friend.
    pub nickname: String,
    /// Gender label; common values are `male`, `female`, and `unknown`, and the
    /// exact values are defined by the upstream.
    pub sex: String,
    /// Age in years.
    pub age: i64,
    /// QID of the friend, supplied by the upstream.
    pub qid: String,
    /// Recommendation reason or brief description.
    pub brief: String,
    /// School information, supplied by the upstream.
    pub school: String,
}

impl BotContext {
    /// Requests the recommended-friend list and waits up to `timeout` for the
    /// response data.
    ///
    /// `timeout` limits only the response wait after the request has been
    /// written to the connection. It does not cover JSON serialization, internal
    /// lock acquisition, or the WebSocket write. The order of the returned list
    /// is determined by the upstream; this method does not sort it.
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
    /// be deserialized into `Vec<RecommendedFriend>`.
    ///
    /// # Cancellation
    ///
    /// Dropping the pending future stops the local wait and removes the matching
    /// echo registration; the request has already been written to the
    /// connection, so cancellation does not withdraw it from the upstream.
    pub async fn get_recommended_friends_with_timeout(
        &self,
        timeout: Duration,
    ) -> Result<Vec<RecommendedFriend>> {
        let response = self
            .websocket_send("get_recommended_friends", serde_json::Value::Null)
            .await?;
        let data = response.data(timeout).await?;
        let friends: Vec<RecommendedFriend> = serde_json::from_value(data)?;
        Ok(friends)
    }

    /// Gets the recommended-friend list using a 10-second response wait.
    ///
    /// Except for the fixed wait duration, the arguments, return value, and
    /// error semantics are the same as
    /// [`Self::get_recommended_friends_with_timeout`].
    pub async fn get_recommended_friends(&self) -> Result<Vec<RecommendedFriend>> {
        self.get_recommended_friends_with_timeout(Duration::from_secs(10))
            .await
    }
}

