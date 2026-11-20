//! Wrapper for the OneBot `set_group_card` action. Methods in this module only send
//! requests and do not wait for an upstream response.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_json::json;

impl BotContext {
    /// Requests that the group card of the specified group member be set.
    ///
    /// This method only sends a `set_group_card` request and does not wait for an upstream
    /// response. `Ok(())` only means the request was written to the current WebSocket
    /// connection; it does not mean the group card has been changed.
    ///
    /// # Arguments
    ///
    /// - `group_id`: group number.
    /// - `user_id`: QQ number of the target group member. This library does not check
    ///   whether the member is in the group.
    /// - `card`: new group card, sent as-is. An empty string usually clears the group card;
    ///   the exact behavior is interpreted by the upstream implementation.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when no usable connection is available,
    /// and [`crate::error::Error::WebSocketError`] when the WebSocket write fails. Upstream
    /// business errors such as insufficient permissions are not returned by this method.
    pub async fn set_group_card(&self, group_id: i64, user_id: i64, card: &str) -> Result<()> {
        let msg = json!(
            {
                "group_id": group_id,
                "user_id": user_id,
                "card": card,
            }
        );
        self.websocket_send("set_group_card", msg).await?;
        Ok(())
    }
}
