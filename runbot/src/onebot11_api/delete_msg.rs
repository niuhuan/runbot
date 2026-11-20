//! Recalls a message.
//!
//! Sends a `delete_msg` request and returns a response handle without waiting for the business
//! result.

use crate::error::Result;
use crate::prelude::BotContext;
use crate::prelude::EchoAsyncResponse;
use serde_json::json;

impl BotContext {
    /// Sends a recall-message request and returns a handle for waiting on the response.
    ///
    /// Returning `Ok` only means the request was written to the current connection; it does
    /// not mean the message has been recalled. Failures caused by permissions or time limits
    /// must be obtained by waiting on the handle.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when no connection is available, or
    /// [`crate::error::Error::WebSocketError`] when the WebSocket send fails.
    pub async fn delete_msg(&self, message_id: i64) -> Result<EchoAsyncResponse> {
        let msg = json!(
            {
                "message_id": message_id,
            }
        );
        self.websocket_send("delete_msg", msg).await
    }
}
