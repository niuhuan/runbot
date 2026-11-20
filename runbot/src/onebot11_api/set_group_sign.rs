//! Group check-in.
//!
//! The two methods send `set_group_sign` and `send_group_sign` respectively and both return a
//! response handle; which action the upstream supports depends on the implementation.

use crate::error::Result;
use crate::prelude::BotContext;
use crate::prelude::EchoAsyncResponse;
use serde_json::json;

impl BotContext {
    /// Sends a group check-in request (action `set_group_sign`) and returns a handle for waiting
    /// on the response.
    ///
    /// Returning `Ok` only means the request has been sent; whether check-in succeeded must be
    /// obtained by waiting on the handle.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when no connection is available, or
    /// [`crate::error::Error::WebSocketError`] when the WebSocket send fails.
    pub async fn set_group_sign(&self, group_id: i64) -> Result<EchoAsyncResponse> {
        let msg = json!(
            {
                "group_id": group_id,
            }
        );
        self.websocket_send("set_group_sign", msg).await
    }
    /// Sends a group check-in request (action `send_group_sign`) and returns a handle for
    /// waiting on the response.
    ///
    /// The parameters are the same as [`Self::set_group_sign`]; only the action name differs.
    /// Whether the upstream supports it depends on the implementation. Returning `Ok` only
    /// means the request has been sent.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when no connection is available, or
    /// [`crate::error::Error::WebSocketError`] when the WebSocket send fails.
    pub async fn send_group_sign(&self, group_id: i64) -> Result<EchoAsyncResponse> {
        let msg = json!(
            {
                "group_id": group_id,
            }
        );
        self.websocket_send("send_group_sign", msg).await
    }
}
