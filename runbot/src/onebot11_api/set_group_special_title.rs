//! Wrapper for the OneBot `set_group_special_title` action. Methods in this module only
//! send requests and do not wait for an upstream response.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_json::json;

impl BotContext {
    /// Requests that the special title of the specified group member be set.
    ///
    /// This method only sends a `set_group_special_title` request and does not wait for an
    /// upstream response. `Ok(())` only means the request was written to the current
    /// WebSocket connection; it does not mean the title has taken effect. This operation
    /// usually requires appropriate group management permissions for the bot.
    ///
    /// # Arguments
    ///
    /// - `group_id`: group number.
    /// - `user_id`: QQ number of the target group member.
    /// - `special_title`: new special title, sent as-is. An empty string usually removes
    ///   the title; the exact behavior is interpreted by the upstream implementation.
    ///   Length and content restrictions are validated by the upstream implementation.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when no usable connection is available,
    /// and [`crate::error::Error::WebSocketError`] when the WebSocket write fails. Upstream
    /// business errors such as insufficient permissions are not returned by this method.
    pub async fn set_group_special_title(
        &self,
        group_id: i64,
        user_id: i64,
        special_title: &str,
    ) -> Result<()> {
        let msg = json!(
            {
                "group_id": group_id,
                "user_id": user_id,
                "special_title": special_title
            }
        );
        self.websocket_send("set_group_special_title", msg).await?;
        Ok(())
    }
}
