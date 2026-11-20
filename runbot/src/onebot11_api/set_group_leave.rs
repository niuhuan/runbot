//! Wrapper for the OneBot `set_group_leave` action. Methods in this module only send
//! requests and do not wait for an upstream response.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_json::json;

impl BotContext {
    /// Requests that the bot leave the specified group or dismiss the group.
    ///
    /// This method only sends a `set_group_leave` request and does not wait for an upstream
    /// response. `Ok(())` only means the request was written to the current WebSocket
    /// connection; it does not mean the bot has left or the group has been dismissed.
    ///
    /// # Arguments
    ///
    /// - `group_id`: target group number.
    /// - `is_dismiss`: when `true`, requests dismissal of the group instead of only leaving
    ///   it. Dismissal usually requires the bot to be the group owner. `is_dismiss` is an
    ///   upstream extension field; implementations that do not support it may ignore it or
    ///   reject the request.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when no usable connection is available,
    /// and [`crate::error::Error::WebSocketError`] when the WebSocket write fails. Upstream
    /// business errors such as insufficient permissions are not returned by this method.
    pub async fn set_group_leave(&self, group_id: i64, is_dismiss: bool) -> Result<()> {
        let msg = json!(
            {
                "group_id": group_id,
                "is_dismiss": is_dismiss,
            }
        );
        self.websocket_send("set_group_leave", msg).await?;
        Ok(())
    }
}
