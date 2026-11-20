//! Wraps the OneBot action `send_group_notice`.
//!
//! The method writes the group notice content to the current WebSocket connection
//! and returns without waiting for the upstream's business response.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_json::json;

impl BotContext {
    /// Publishes a group notice.
    ///
    /// `Ok(())` means only that the request has been written to the current
    /// WebSocket; it does not mean the notice has been published. Administrator
    /// permissions and content review results are not awaited.
    ///
    /// # Arguments
    ///
    /// - `group_id`: number of the group where the notice will be published.
    /// - `content`: notice content, interpreted in the format supported by the
    ///   upstream; this library does not parse, escape, or validate it.
    ///
    /// # Notes
    ///
    /// The current wrapper sends only `group_id` and `content`. Some upstream
    /// extensions also support optional fields such as `image`, which have no
    /// corresponding argument in this method. To send those fields, call
    /// [`crate::bot_context::BotContext::websocket_send`] directly to build the
    /// complete request.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when the current connection is
    /// unavailable (not connected, disconnected, or closed);
    /// [`crate::error::Error::WebSocketError`] when the WebSocket write fails.
    /// Upstream permission or content validation failures are not awaited or
    /// returned by this method.
    pub async fn send_group_notice(&self, group_id: i64, content: &str) -> Result<()> {
        self.websocket_send(
            "send_group_notice",
            json!({
                "group_id": group_id,
                "content": content,
            }),
        )
        .await?;
        Ok(())
    }
}

