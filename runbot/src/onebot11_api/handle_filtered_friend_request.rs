//! Wraps the OneBot action `handle_filtered_friend_request`.
//!
//! The method writes the parameters for handling a filtered friend request to
//! the current WebSocket connection and returns without waiting for the
//! upstream's business response.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_json::json;

impl BotContext {
    /// Handles a filtered friend request by approving or rejecting it.
    ///
    /// An available connection should be established before calling this method.
    /// `Ok(())` means only that the request has been written to the current
    /// WebSocket; it does not mean the upstream has accepted the request or that
    /// the friend relationship has changed.
    ///
    /// # Arguments
    ///
    /// - `request_id`: identifier of the filtered friend request, usually from
    ///   the `request_id` field of a
    ///   [`crate::onebot11_api::get_filtered_friend_requests::FilteredFriendRequest`].
    /// - `approve`: `true` to approve, `false` to reject.
    /// - `remark`: remark to use when approving. When `Some`, it is written to
    ///   the `remark` field as a string; when `None`, the key is omitted from
    ///   `params` entirely and JSON `null` is not sent.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when the current connection is
    /// unavailable (not connected, disconnected, or closed);
    /// [`crate::error::Error::WebSocketError`] when the WebSocket write fails.
    /// Permission or parameter validation failures from the upstream are not
    /// awaited or returned by this method.
    pub async fn handle_filtered_friend_request(
        &self,
        request_id: i64,
        approve: bool,
        remark: Option<&str>,
    ) -> Result<()> {
        let mut params = json!({
            "request_id": request_id,
            "approve": approve,
        });
        if let Some(r) = remark {
            params["remark"] = json!(r);
        }
        self.websocket_send("handle_filtered_friend_request", params)
            .await?;
        Ok(())
    }
}

