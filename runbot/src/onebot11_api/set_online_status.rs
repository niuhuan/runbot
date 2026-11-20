//! Wrapper for the OneBot `set_online_status` action. Methods in this module only send
//! requests and do not wait for an upstream response.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_json::json;

impl BotContext {
    /// Requests that the current account's online status be set.
    ///
    /// This action is an upstream implementation extension, not part of the OneBot 11
    /// standard API, and support depends on the upstream implementation. This method only
    /// sends a request and does not wait for a response. `Ok(())` only means the request
    /// was written to the current WebSocket connection; it does not mean the online status
    /// has changed.
    ///
    /// # Arguments
    ///
    /// - `status`: online status number, sent as-is. Values and meaning are defined by the
    ///   upstream implementation; this library does not validate them.
    ///
    /// # Notes
    ///
    /// The current implementation only sends `status`. Some upstreams also accept fields
    /// such as extended status and battery level for this action; this method does not send
    /// them, so the upstream may reject the request.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when no usable connection is available,
    /// and [`crate::error::Error::WebSocketError`] when the WebSocket write fails. If the
    /// upstream does not recognize the action or rejects the parameters, this method does
    /// not return the corresponding error.
    pub async fn set_online_status(&self, status: i32) -> Result<()> {
        self.websocket_send(
            "set_online_status",
            json!({
                "status": status,
            }),
        )
        .await?;
        Ok(())
    }
}

