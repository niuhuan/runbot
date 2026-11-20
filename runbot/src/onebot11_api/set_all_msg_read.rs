//! Wrapper for the OneBot `set_all_msg_read` action. Methods in this module only send
//! requests and do not wait for an upstream response.

use crate::error::Result;
use crate::prelude::BotContext;

impl BotContext {
    /// Requests that all messages for the current account be marked as read.
    ///
    /// This action is an upstream implementation extension, and upstream support varies.
    /// The method sends a `set_all_msg_read` request without parameters and returns
    /// immediately without waiting for an upstream response. `Ok(())` only means the
    /// request was written to the current WebSocket connection; it does not mean any
    /// message was marked as read.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when no usable connection is available,
    /// and [`crate::error::Error::WebSocketError`] when the WebSocket write fails. Upstream
    /// business errors are not returned by this method.
    pub async fn set_all_msg_read(&self) -> Result<()> {
        self.websocket_send("set_all_msg_read", serde_json::Value::Null)
            .await?;
        Ok(())
    }
}

