//! Wrapper for the OneBot `set_group_whole_ban` action. Methods in this module only send
//! requests and do not wait for an upstream response.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_json::json;

impl BotContext {
    /// Requests that all members of the specified group be muted or unmuted.
    ///
    /// This method only sends a `set_group_whole_ban` request and does not wait for an
    /// upstream response. `Ok(())` only means the request was written to the current
    /// WebSocket connection; it does not mean the mute state has changed. This operation
    /// usually requires appropriate group management permissions for the bot.
    ///
    /// # Arguments
    ///
    /// - `group_id`: group number.
    /// - `enable`: `true` mutes all members; `false` unmutes them.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when no usable connection is available,
    /// and [`crate::error::Error::WebSocketError`] when the WebSocket write fails. Upstream
    /// business errors such as insufficient permissions are not returned by this method.
    pub async fn set_group_whole_ban(&self, group_id: i64, enable: bool) -> Result<()> {
        let msg = json!(
            {
                "group_id": group_id,
                "enable": enable,
            }
        );
        self.websocket_send("set_group_whole_ban", msg).await?;
        Ok(())
    }
}
