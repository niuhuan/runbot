//! Handle join-group or group-invitation requests.
//!
//! Sends a `set_group_add_request` request and returns a response handle without waiting
//! for the business result.

use crate::error::Result;
use crate::prelude::BotContext;
use crate::prelude::EchoAsyncResponse;
use serde_json::json;

impl BotContext {
    /// Send a command to handle a group request and return a handle for waiting on the
    /// response.
    ///
    /// An `Ok` return only means the request has been written to the current connection; it
    /// does not mean the group request has been handled.
    ///
    /// # Arguments
    ///
    /// - `flag`: Request identifier, usually from [`crate::event::GroupRequest::flag`].
    /// - `approve`: `true` to approve, `false` to reject.
    /// - `reason`: Rejection reason; `None` is written as an empty string. The current
    ///   implementation places this value in the request body's `remark` field rather than
    ///   the `reason` field commonly used by the OneBot standard; whether the upstream
    ///   accepts it depends on its implementation.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when the current connection is
    /// unavailable; returns [`crate::error::Error::WebSocketError`] when the WebSocket send
    /// fails.
    pub async fn set_group_add_request(
        &self,
        flag: &str,
        approve: bool,
        reason: impl Into<Option<&str>>,
    ) -> Result<EchoAsyncResponse> {
        let msg = json!(
            {
                "flag": flag,
                "approve": approve,
                "remark": if let Some(r) = reason.into() { r } else { "" },
            }
        );
        self.websocket_send("set_group_add_request", msg).await
    }
}
