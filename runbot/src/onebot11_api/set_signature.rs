//! Wrapper for the OneBot `set_signature` action. Methods in this module only send
//! requests and do not wait for an upstream response.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_json::json;

impl BotContext {
    /// Requests that the current account's signature be set.
    ///
    /// This action is an upstream implementation extension, not part of the OneBot 11
    /// standard API, and support depends on the upstream implementation. This method only
    /// sends a request and does not wait for a response. `Ok(())` only means the request
    /// was written to the current WebSocket connection; it does not mean the signature has
    /// been changed.
    ///
    /// # Arguments
    ///
    /// - `signature`: new signature, sent as-is. Whether an empty string clears the
    ///   signature is interpreted by the upstream implementation. Length and content
    ///   restrictions are validated by the upstream implementation.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when no usable connection is available,
    /// and [`crate::error::Error::WebSocketError`] when the WebSocket write fails. If the
    /// upstream does not recognize the action or rejects the parameters, this method does
    /// not return the corresponding error.
    pub async fn set_signature(&self, signature: &str) -> Result<()> {
        self.websocket_send(
            "set_signature",
            json!({
                "signature": signature,
            }),
        )
        .await?;
        Ok(())
    }
}

