//! Clicks a button in a message (upstream extension action `click_button`).
//!
//! This module only sends the request and does not wait for a OneBot response.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_json::json;

impl BotContext {
    /// Sends a request to click a button in the specified message.
    ///
    /// This method does not wait for a OneBot response. Returning `Ok(())` only means the
    /// request was written to the current WebSocket connection; it does not mean the button
    /// was clicked or the callback was triggered.
    ///
    /// # Arguments
    ///
    /// - `message_id`: ID of the message containing the button.
    /// - `button_id`: button identifier. The encoding and allowed values are determined by
    ///   the button definition in the message and by the upstream implementation. This
    ///   method does not validate that the button exists.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when no connection is available, or
    /// [`crate::error::Error::WebSocketError`] when the WebSocket write fails. Business
    /// errors such as a missing or expired button, or upstream rejection, are not waited for
    /// or returned by this method.
    pub async fn click_button(&self, message_id: i64, button_id: &str) -> Result<()> {
        self.websocket_send(
            "click_button",
            json!({
                "message_id": message_id,
                "button_id": button_id,
            }),
        )
        .await?;
        Ok(())
    }
}

