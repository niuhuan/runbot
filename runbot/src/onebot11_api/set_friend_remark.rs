//! Wrapper for the OneBot `set_friend_remark` action. Methods in this module only send
//! requests and do not wait for an upstream response.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_json::json;

impl BotContext {
    /// Requests that the current account's remark for the specified friend be set.
    ///
    /// This action is an upstream implementation extension, not part of the OneBot 11
    /// standard API, and support depends on the upstream implementation. This method only
    /// sends a request and does not wait for a response. `Ok(())` only means the request
    /// was written to the current WebSocket connection; it does not mean the remark has
    /// been changed.
    ///
    /// # Arguments
    ///
    /// - `user_id`: friend's QQ number.
    /// - `remark`: new friend remark, sent as-is. Whether an empty string clears the remark
    ///   is interpreted by the upstream implementation.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when no usable connection is available,
    /// and [`crate::error::Error::WebSocketError`] when the WebSocket write fails. Upstream
    /// business errors such as rejected parameters or a missing friend are not returned by
    /// this method.
    pub async fn set_friend_remark(&self, user_id: i64, remark: &str) -> Result<()> {
        self.websocket_send(
            "set_friend_remark",
            json!({
                "user_id": user_id,
                "remark": remark,
            }),
        )
        .await?;
        Ok(())
    }
}

