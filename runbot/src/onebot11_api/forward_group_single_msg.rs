//! Forwards a single message to a group (upstream extension action
//! `forward_group_single_msg`).
//!
//! The method sends the request and returns a response handle; a wait method on the handle
//! must be called to confirm the upstream result. See the method documentation for a limitation
//! in the current target parameter.

use crate::error::Result;
use crate::prelude::BotContext;
use crate::prelude::EchoAsyncResponse;
use serde_json::json;

impl BotContext {
    /// Sends a request to forward the specified message to a group and returns a handle for
    /// waiting on the response.
    ///
    /// Returning `Ok` only means the request was written to the current WebSocket; it does
    /// not mean the message has been forwarded. The current implementation sends
    /// `message_id` and `user_id` in `params` and does not send `group_id`, so this method
    /// cannot specify the target group through its parameters. If the upstream requires
    /// `group_id`, the request may fail or may not reach the intended group; callers should
    /// confirm the parameter contract of the upstream they use.
    ///
    /// # Arguments
    ///
    /// - `message_id`: ID of the existing message to forward.
    /// - `user_id`: QQ ID sent in the `user_id` field; this method does not interpret or
    ///   validate it as a group ID.
    ///
    /// # Returns
    ///
    /// Returns a handle for waiting on the response. The structure of the response `data` is
    /// determined by the upstream implementation, and this module does not define a concrete
    /// type. Call [`crate::bot_context::EchoAsyncResponse::ok`] to check the business status,
    /// or [`crate::bot_context::EchoAsyncResponse::data`] to read the raw data.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when no connection is available, or
    /// [`crate::error::Error::WebSocketError`] when the WebSocket write fails. Business
    /// errors such as upstream rejection, a parameter mismatch, or a non-zero `retcode`
    /// appear when waiting on the handle; this method does not wait.
    ///
    /// # Cancellation
    ///
    /// Dropping the returned handle removes the local echo registration but does not revoke
    /// the forwarding request already sent. A timeout or cancellation does not prove the
    /// upstream did not execute it.
    pub async fn forward_group_single_msg(
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
        self.websocket_send("forward_group_single_msg", msg).await
    }
}
