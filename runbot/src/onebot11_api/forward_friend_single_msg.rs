//! Forwards a single message to a friend (upstream extension action
//! `forward_friend_single_msg`).
//!
//! The method sends the request and returns a response handle; a wait method on the handle
//! must be called to confirm the upstream result.

use crate::error::Result;
use crate::prelude::BotContext;
use crate::prelude::EchoAsyncResponse;
use serde_json::json;

impl BotContext {
    /// Sends a request to forward the specified message to a friend and returns a handle for
    /// waiting on the response.
    ///
    /// Returning `Ok` only means the request was written to the current WebSocket; it does
    /// not mean the message has been forwarded. The structure of the response `data` is
    /// determined by the upstream implementation, and this module does not define a concrete
    /// type. To check only the business status, call
    /// [`crate::bot_context::EchoAsyncResponse::ok`]; to read the raw forwarding result, call
    /// [`crate::bot_context::EchoAsyncResponse::data`].
    ///
    /// # Arguments
    ///
    /// - `message_id`: ID of the existing message to forward.
    /// - `user_id`: QQ ID of the friend receiving the forwarded message.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when no connection is available, or
    /// [`crate::error::Error::WebSocketError`] when the WebSocket write fails. Business
    /// errors such as upstream rejection, a missing message, or a non-zero `retcode` appear
    /// when waiting on the handle; this method does not wait.
    ///
    /// # Cancellation
    ///
    /// Dropping the returned handle removes the local echo registration but does not revoke
    /// the forwarding request already sent. A timeout or cancellation does not prove the
    /// upstream did not execute it.
    pub async fn forward_friend_single_msg(
        &self,
        message_id: i64,
        user_id: i64,
    ) -> Result<EchoAsyncResponse> {
        let msg = json!(
            {
                "message_id": message_id,
                "user_id": user_id,
            }
        );
        self.websocket_send("forward_friend_single_msg", msg).await
    }
}
