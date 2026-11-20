//! Handle friend add requests.
//!
//! Sends a `set_friend_add_request` request and returns a response handle without waiting
//! for the business result.

use crate::error::Result;
use crate::prelude::BotContext;
use crate::prelude::EchoAsyncResponse;
use serde_json::json;

impl BotContext {
    /// Send a command to handle a friend add request and return a handle for waiting on the
    /// response.
    ///
    /// An `Ok` return only means the request has been written to the current connection; it
    /// does not mean the friend request has been handled.
    ///
    /// # Arguments
    ///
    /// - `flag`: Request identifier, usually from [`crate::event::FriendRequest::flag`].
    /// - `approve`: `true` to approve, `false` to reject.
    /// - `remark`: Friend remark to set when approving; `None` is sent as JSON `null`, and
    ///   whether the upstream accepts it depends on the implementation.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when the current connection is
    /// unavailable; returns [`crate::error::Error::WebSocketError`] when the WebSocket send
    /// fails.
    pub async fn set_friend_add_request(
        &self,
        flag: &str,
        approve: bool,
        remark: Option<&str>,
    ) -> Result<EchoAsyncResponse> {
        let msg = json!(
            {
                "flag": flag,
                "approve": approve,
                "remark": remark,
            }
        );
        self.websocket_send("set_friend_add_request", msg).await
    }
}
