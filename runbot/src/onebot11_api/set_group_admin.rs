//! Wrapper for the OneBot `set_group_admin` action. Methods in this module only send
//! requests and do not wait for an upstream response.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_json::json;

impl BotContext {
    /// Requests that the administrator status of the specified group member be set or
    /// removed.
    ///
    /// This method only sends a `set_group_admin` request and does not wait for an upstream
    /// response. `Ok(())` only means the request was written to the current WebSocket
    /// connection; it does not mean the administrator status has changed. This operation
    /// usually requires the bot to be the group owner.
    ///
    /// # Arguments
    ///
    /// - `group_id`: group number.
    /// - `user_id`: QQ number of the target group member.
    /// - `enable`: `true` grants administrator status; `false` removes it.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when no usable connection is available,
    /// and [`crate::error::Error::WebSocketError`] when the WebSocket write fails. Upstream
    /// business errors such as insufficient permissions are not returned by this method.
    pub async fn set_group_admin(&self, group_id: i64, user_id: i64, enable: bool) -> Result<()> {
        let msg = json!(
            {
                "group_id": group_id,
                "user_id": user_id,
                "enable": enable,
            }
        );
        self.websocket_send("set_group_admin", msg).await?;
        Ok(())
    }
}
