//! Wrapper for the OneBot `set_private_msg_read` action. Methods in this module only send
//! requests and do not wait for an upstream response.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_json::json;

impl BotContext {
    /// Requests that private messages with the specified user be marked as read.
    ///
    /// This action is an upstream implementation extension, not part of the OneBot 11
    /// standard API, and support depends on the upstream implementation. This method only
    /// sends a request and does not wait for a response. `Ok(())` only means the request
    /// was written to the current WebSocket connection; it does not mean the messages have
    /// been marked as read.
    ///
    /// # Arguments
    ///
    /// - `user_id`: QQ number of the other party in the private chat.
    /// - `message_id`: target message ID. The exact range of messages marked as read is
    ///   interpreted by the upstream implementation.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when no usable connection is available,
    /// and [`crate::error::Error::WebSocketError`] when the WebSocket write fails. If the
    /// upstream does not recognize the action or rejects the parameters, this method does
    /// not return the corresponding error.
    pub async fn set_private_msg_read(&self, user_id: i64, message_id: i64) -> Result<()> {
        self.websocket_send(
            "set_private_msg_read",
            json!({
                "user_id": user_id,
                "message_id": message_id,
            }),
        )
        .await?;
        Ok(())
    }
}

