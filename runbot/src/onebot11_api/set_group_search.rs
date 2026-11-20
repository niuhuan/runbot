//! Wrapper for the OneBot `set_group_search` action. Methods in this module only send
//! requests and do not wait for an upstream response.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_json::json;

impl BotContext {
    /// Requests that the specified group be made searchable or non-searchable.
    ///
    /// This action is an upstream implementation extension, not part of the OneBot 11
    /// standard API, and support depends on the upstream implementation. This method only
    /// sends a request and does not wait for a response. `Ok(())` only means the request
    /// was written to the current WebSocket connection; it does not mean the search setting
    /// has taken effect.
    ///
    /// # Arguments
    ///
    /// - `group_id`: target group number.
    /// - `enable`: `true` allows the group to be found through search; `false` disallows
    ///   it. The exact semantics are interpreted by the upstream implementation.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when no usable connection is available,
    /// and [`crate::error::Error::WebSocketError`] when the WebSocket write fails. If the
    /// upstream does not recognize the action or rejects the parameters, this method does
    /// not return the corresponding error.
    pub async fn set_group_search(&self, group_id: i64, enable: bool) -> Result<()> {
        self.websocket_send(
            "set_group_search",
            json!({
                "group_id": group_id,
                "enable": enable,
            }),
        )
        .await?;
        Ok(())
    }
}

