//! Removes an essence message (OneBot 11 action `delete_essence_msg`).
//!
//! This module only sends the request and does not wait for a OneBot response.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_json::json;

impl BotContext {
    /// Sends a request to remove the specified message from the group essence message list.
    ///
    /// This method does not wait for a OneBot response. Returning `Ok(())` only means the
    /// request was written to the current WebSocket connection; it does not mean the message
    /// has been removed from the essence message list.
    ///
    /// # Arguments
    ///
    /// - `message_id`: ID of the message to remove from the essence list.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when no connection is available, or
    /// [`crate::error::Error::WebSocketError`] when the WebSocket write fails. Business
    /// errors such as insufficient caller permissions, the message not being in the essence
    /// list, or upstream rejection are not waited for or returned by this method.
    pub async fn delete_essence_msg(&self, message_id: i64) -> Result<()> {
        self.websocket_send(
            "delete_essence_msg",
            json!({
                "message_id": message_id,
            }),
        )
        .await?;
        Ok(())
    }
}

