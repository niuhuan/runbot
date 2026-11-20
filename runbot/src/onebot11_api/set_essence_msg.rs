//! Wrapper for the OneBot `set_essence_msg` action. Methods in this module only send
//! requests and do not wait for an upstream response.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_json::json;

impl BotContext {
    /// Requests that the specified group message be set as a group essence message.
    ///
    /// This method only sends a `set_essence_msg` request and does not wait for an upstream
    /// response. `Ok(())` only means the request was written to the current WebSocket
    /// connection; it does not mean the message has become an essence message. This
    /// operation usually requires appropriate group management permissions for the bot.
    ///
    /// # Arguments
    ///
    /// - `message_id`: message ID of the target group message. This library does not check
    ///   whether the message exists or which group it belongs to.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when no usable connection is available,
    /// and [`crate::error::Error::WebSocketError`] when the WebSocket write fails. Upstream
    /// business errors such as insufficient permissions are not returned by this method.
    pub async fn set_essence_msg(&self, message_id: i64) -> Result<()> {
        self.websocket_send(
            "set_essence_msg",
            json!({
                "message_id": message_id,
            }),
        )
        .await?;
        Ok(())
    }
}

