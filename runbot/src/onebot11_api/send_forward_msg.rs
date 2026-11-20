//! Sends merged forward messages.
//!
//! Both methods return a [`crate::bot_context::EchoAsyncResponse`] handle; a successful send
//! does not mean the message has been delivered.

use crate::error::Result;
use crate::prelude::BotContext;
use crate::prelude::EchoAsyncResponse;
use serde_json::json;

impl BotContext {
    /// Sends a merged forward message to a friend and returns a handle for waiting on the
    /// response.
    ///
    /// # Arguments
    ///
    /// - `user_id`: QQ ID of the recipient.
    /// - `messages`: array of OneBot merged forward nodes, placed into the request as is; this
    ///   method does not validate the node structure.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when no connection is available, or
    /// [`crate::error::Error::WebSocketError`] when the WebSocket send fails. Upstream business
    /// errors must be obtained by waiting on the handle.
    pub async fn send_private_forward_msg(
        &self,
        user_id: i64,
        messages: serde_json::Value,
    ) -> Result<EchoAsyncResponse> {
        self.websocket_send(
            "send_private_forward_msg",
            json!({
                "user_id": user_id,
                "messages": messages,
            }),
        )
        .await
    }

    /// Sends a merged forward message to a group and returns a handle for waiting on the
    /// response.
    ///
    /// # Arguments
    ///
    /// - `group_id`: target group ID.
    /// - `messages`: array of OneBot merged forward nodes, placed into the request as is; this
    ///   method does not validate the node structure.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when no connection is available, or
    /// [`crate::error::Error::WebSocketError`] when the WebSocket send fails. Upstream business
    /// errors must be obtained by waiting on the handle.
    pub async fn send_group_forward_msg(
        &self,
        group_id: i64,
        messages: serde_json::Value,
    ) -> Result<EchoAsyncResponse> {
        self.websocket_send(
            "send_group_forward_msg",
            json!({
                "group_id": group_id,
                "messages": messages,
            }),
        )
        .await
    }
}

