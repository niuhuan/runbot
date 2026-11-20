//! Kicks a group member.
//!
//! Sends a `set_group_kick` request and returns a response handle without waiting for the
//! business result.

use crate::error::Result;
use crate::prelude::BotContext;
use crate::prelude::EchoAsyncResponse;
use serde_json::json;

impl BotContext {
    /// Sends a request to kick a group member and returns a handle for waiting on the response.
    ///
    /// Returning `Ok` only means the request was written to the current connection; it does
    /// not mean the member has been removed from the group.
    ///
    /// # Arguments
    ///
    /// - `group_id`: target group ID.
    /// - `user_id`: QQ ID of the member to kick.
    /// - `reject_add_request`: whether to also reject this member's future join requests;
    ///   whether this takes effect depends on the upstream.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when no connection is available, or
    /// [`crate::error::Error::WebSocketError`] when the WebSocket send fails.
    pub async fn set_group_kick(
        &self,
        group_id: i64,
        user_id: i64,
        reject_add_request: bool,
    ) -> Result<EchoAsyncResponse> {
        let msg = json!(
            {
                "group_id": group_id,
                "user_id": user_id,
                "reject_add_request": reject_add_request,
            }
        );
        self.websocket_send("set_group_kick", msg).await
    }
}
