//! Creates a favorite (upstream extension action `create_favorite`).
//!
//! This module only sends the request and does not wait for a OneBot response.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_json::json;

impl BotContext {
    /// Sends a request to create a favorite from the specified message.
    ///
    /// This method does not wait for a OneBot response. Returning `Ok(())` only means the
    /// request was written to the current WebSocket connection; it does not mean the message
    /// has been favorited. The destination, deduplication rules, and permission requirements
    /// are determined by the upstream implementation.
    ///
    /// # Arguments
    ///
    /// - `message_id`: ID of the message to favorite.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when no connection is available, or
    /// [`crate::error::Error::WebSocketError`] when the WebSocket write fails. Business
    /// errors such as a missing message, insufficient access, or upstream rejection are not
    /// waited for or returned by this method.
    pub async fn create_favorite(&self, message_id: i64) -> Result<()> {
        self.websocket_send(
            "create_favorite",
            json!({
                "message_id": message_id,
            }),
        )
        .await?;
        Ok(())
    }
}

