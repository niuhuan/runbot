//! Wrapper for the OneBot `set_online_model` action. Methods in this module only send
//! requests and do not wait for an upstream response.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_json::json;

impl BotContext {
    /// Requests that the current account's online device model be set.
    ///
    /// This action is an upstream implementation extension, not part of the OneBot 11
    /// standard API, and support depends on the upstream implementation. This method only
    /// sends a request and does not wait for a response. `Ok(())` only means the request
    /// was written to the current WebSocket connection; it does not mean the online device
    /// model has been changed.
    ///
    /// # Arguments
    ///
    /// - `model`: device model name or identifier, sent as-is. Available values are defined
    ///   by the upstream implementation; this library does not validate them.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when no usable connection is available,
    /// and [`crate::error::Error::WebSocketError`] when the WebSocket write fails. If the
    /// upstream does not recognize the action or rejects the parameters, this method does
    /// not return the corresponding error.
    pub async fn set_online_model(&self, model: &str) -> Result<()> {
        self.websocket_send(
            "set_online_model",
            json!({
                "model": model,
            }),
        )
        .await?;
        Ok(())
    }
}

