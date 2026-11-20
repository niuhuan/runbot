//! Fetches the friend list.
//!
//! `get_friend_list` waits up to ten seconds for a response and returns the friend data;
//! `get_friend_list_with_timeout` allows a custom response wait duration.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use tokio::time::Duration;

/// An entry in the friend list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Friend {
    /// QQ ID of the friend.
    pub user_id: i64,
    /// Friend nickname.
    pub nickname: String,
    /// The remark set for this friend by the current account.
    ///
    /// A missing field deserializes to an empty string; JSON `null` is not automatically
    /// converted to an empty string. An empty value does not indicate a failed request;
    /// whether to fall back to [`Self::nickname`] for display is left to the caller.
    #[serde(default)]
    pub remark: String,
}

impl BotContext {
    /// Requests the friend list and waits up to the given time for the response data.
    ///
    /// A usable connection must already be established before calling this method. `timeout`
    /// limits the response wait after the request has been sent; it does not include the send
    /// phase. This method does not retry on its own and does not reorder the returned list.
    ///
    /// # Errors
    ///
    /// - Returns [`crate::error::Error::StateError`] when the current connection is
    ///   unavailable or the response channel is closed.
    /// - Returns [`crate::error::Error::WebSocketError`] when the WebSocket send fails.
    /// - Returns [`crate::error::Error::TimeoutError`] when no response arrives within
    ///   `timeout`.
    /// - Returns [`crate::error::Error::StateError`] when the upstream response has a
    ///   non-zero `retcode`.
    /// - Returns [`crate::error::Error::JsonError`] when the response data cannot be
    ///   deserialized into a friend list.
    ///
    /// # Cancellation
    ///
    /// Dropping the pending future ends the local wait and releases the corresponding
    /// response registration; the request already sent is not revoked from the upstream for
    /// this reason.
    ///
    /// # Examples
    ///
    /// The following function takes a connected context and sets a five-second timeout for
    /// the response wait.
    ///
    /// ```no_run
    /// use runbot::bot_context::BotContext;
    /// use std::time::Duration;
    ///
    /// async fn print_friends(bot: &BotContext) -> runbot::error::Result<()> {
    ///     for friend in bot.get_friend_list_with_timeout(Duration::from_secs(5)).await? {
    ///         println!("{}: {}", friend.user_id, friend.nickname);
    ///     }
    ///     Ok(())
    /// }
    /// # fn main() {}
    /// ```
    pub async fn get_friend_list_with_timeout(&self, timeout: Duration) -> Result<Vec<Friend>> {
        let response = self
            .websocket_send("get_friend_list", serde_json::Value::Null)
            .await?;
        let response = response.data(timeout).await?;
        let friend_list: Vec<Friend> = serde_json::from_value(response)?;
        Ok(friend_list)
    }

    /// Calls [`Self::get_friend_list_with_timeout`] with a ten-second response wait.
    ///
    /// Timeout, connection, business status code, and deserialization errors match the
    /// timeout variant; the list order is determined by the upstream response.
    pub async fn get_friend_list(&self) -> Result<Vec<Friend>> {
        self.get_friend_list_with_timeout(Duration::from_secs(10))
            .await
    }
}
