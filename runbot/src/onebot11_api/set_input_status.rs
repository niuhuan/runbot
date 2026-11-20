//! Wrapper for the OneBot `set_input_status` action. Methods in this module only send
//! requests and do not wait for an upstream response.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_json::json;

impl BotContext {
    /// Requests that the typing status with the specified user be set (for example,
    /// "typing").
    ///
    /// This action is an upstream implementation extension, not part of the OneBot 11
    /// standard API, and support depends on the upstream implementation. This method only
    /// sends a request and does not wait for a response. `Ok(())` only means the request
    /// was written to the current WebSocket connection; it does not mean the other user
    /// has seen the typing status.
    ///
    /// # Arguments
    ///
    /// - `user_id`: QQ number of the user whose typing status is targeted.
    /// - `typing`: boolean typing status. `true` means typing; `false` means stopped
    ///   typing. The exact semantics are interpreted by the upstream implementation.
    ///
    /// # Notes
    ///
    /// The current implementation sends a field named `typing`. Some upstreams (for
    /// example, NapCat) use a numeric `event_type` field for typing status; if `typing` is
    /// not recognized, the request may be rejected.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when no usable connection is available,
    /// and [`crate::error::Error::WebSocketError`] when the WebSocket write fails. If the
    /// upstream does not recognize the action or field, this method does not return the
    /// corresponding error.
    pub async fn set_input_status(&self, user_id: i64, typing: bool) -> Result<()> {
        self.websocket_send(
            "set_input_status",
            json!({
                "user_id": user_id,
                "typing": typing,
            }),
        )
        .await?;
        Ok(())
    }
}

