//! Wraps the OneBot action `get_unidirectional_friend_list`.
//!
//! The method sends a query request, waits for the response with the matching
//! echo, checks `retcode`, and deserializes `data` into a list of
//! [`UnidirectionalFriend`] values. A unidirectional friend is usually a user
//! who has added the bot as a friend but has not been added back by the bot.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use tokio::time::Duration;

/// One entry in the unidirectional-friend list.
///
/// All three fields are required: a missing field or an explicit `null` causes
/// deserialization to fail with [`crate::error::Error::JsonError`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnidirectionalFriend {
    /// QQ number of the unidirectional friend.
    pub user_id: i64,
    /// Friend nickname.
    pub nickname: String,
    /// Friend source description supplied by the upstream; this library does not
    /// interpret its value.
    pub source: String,
}

impl BotContext {
    /// Requests the unidirectional-friend list and waits up to `timeout` for the
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
    /// be deserialized into `Vec<UnidirectionalFriend>`.
    ///
    /// # Cancellation
    ///
    /// Dropping the pending future stops the local wait and removes the matching
    /// echo registration; the request has already been written to the
    /// connection, so cancellation does not withdraw it from the upstream.
    pub async fn get_unidirectional_friend_list_with_timeout(
        &self,
        timeout: Duration,
    ) -> Result<Vec<UnidirectionalFriend>> {
        let response = self
            .websocket_send("get_unidirectional_friend_list", serde_json::Value::Null)
            .await?;
        let data = response.data(timeout).await?;
        let friends: Vec<UnidirectionalFriend> = serde_json::from_value(data)?;
        Ok(friends)
    }

    /// Gets the unidirectional-friend list using a 10-second response wait.
    ///
    /// Except for the fixed wait duration, the arguments, return value, and
    /// error semantics are the same as
    /// [`Self::get_unidirectional_friend_list_with_timeout`].
    pub async fn get_unidirectional_friend_list(&self) -> Result<Vec<UnidirectionalFriend>> {
        self.get_unidirectional_friend_list_with_timeout(Duration::from_secs(10))
            .await
    }
}

