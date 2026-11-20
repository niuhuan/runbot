//! Deletes a friend.
//!
//! Sends a `delete_friend` request and returns a response handle without waiting for the
//! business result.

use crate::error::Result;
use crate::prelude::BotContext;
use crate::prelude::EchoAsyncResponse;

impl BotContext {
    /// Sends a delete-friend request and returns a handle for waiting on the response.
    ///
    /// Returning `Ok` only means the request was written to the current connection; it does
    /// not mean the friendship has been terminated.
    ///
    /// # Arguments
    ///
    /// - `user_id`: QQ ID of the friend to delete.
    /// - `temp_block`: whether to also temporarily block the user; whether this takes effect
    ///   depends on the upstream.
    /// - `temp_both_del`: whether to also delete the friend in both directions; whether this
    ///   takes effect depends on the upstream (some implementations ignore it).
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when no connection is available, or
    /// [`crate::error::Error::WebSocketError`] when the WebSocket send fails.
    pub async fn delete_friend(
        &self,
        user_id: i64,
        temp_block: bool,
        temp_both_del: bool,
    ) -> Result<EchoAsyncResponse> {
        self.websocket_send(
            "delete_friend",
            serde_json::json!({
                "user_id": user_id,
                "temp_block": temp_block,
                "temp_both_del": temp_both_del,
            }),
        )
        .await
    }
}
