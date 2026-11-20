//! Wraps the OneBot action `send_like`.
//!
//! The method writes the target and count for the like request to the current
//! WebSocket connection and returns without waiting for the upstream's business
//! response.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_json::json;

impl BotContext {
    /// Sends a like request to the specified user.
    ///
    /// `Ok(())` means only that the request has been written to the current
    /// WebSocket; it does not mean the like count has increased. Permissions,
    /// daily limits, and execution results are determined by the upstream, and
    /// this method does not wait for them.
    ///
    /// # Arguments
    ///
    /// - `user_id`: QQ number of the user to like.
    /// - `times`: number of likes to request. Common implementations limit each
    ///   friend to 10 likes per day; this library does not validate the range,
    ///   and the actual limit is determined by the upstream.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when the current connection is
    /// unavailable (not connected, disconnected, or closed);
    /// [`crate::error::Error::WebSocketError`] when the WebSocket write fails.
    /// Upstream permission or rate-limit errors are not awaited or returned by
    /// this method.
    pub async fn send_like(&self, user_id: i64, times: i32) -> Result<()> {
        self.websocket_send(
            "send_like",
            json!({
                "user_id": user_id,
                "times": times,
            }),
        )
        .await?;
        Ok(())
    }
}

