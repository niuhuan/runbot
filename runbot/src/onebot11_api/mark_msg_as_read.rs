//! Wraps the OneBot action `mark_msg_as_read`.
//!
//! The method writes the message ID to the current WebSocket connection and
//! returns without waiting for the upstream's business response.

use crate::error::Result;
use crate::prelude::BotContext;

impl BotContext {
    /// Requests that the specified message be marked as read.
    ///
    /// `Ok(())` means only that the request has been written to the current
    /// WebSocket; it does not mean the upstream has marked the message as read.
    ///
    /// # Arguments
    ///
    /// - `message_id`: ID of the message to mark as read, usually from a message
    ///   event or the response from sending a message.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when the current connection is
    /// unavailable (not connected, disconnected, or closed);
    /// [`crate::error::Error::WebSocketError`] when the WebSocket write fails.
    /// Whether the upstream accepts the message ID is not awaited or returned by
    /// this method.
    pub async fn mark_msg_as_read(&self, message_id: i64) -> Result<()> {
        self.websocket_send(
            "mark_msg_as_read",
            serde_json::json!({
                "message_id": message_id,
            }),
        )
        .await?;
        Ok(())
    }
}
