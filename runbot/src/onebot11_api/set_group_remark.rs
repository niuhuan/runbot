//! Wrapper for the OneBot `set_group_remark` action. Methods in this module only send
//! requests and do not wait for an upstream response.

use crate::error::Result;
use crate::prelude::BotContext;

impl BotContext {
    /// Requests that the current account's remark for the specified group be set.
    ///
    /// This action is an upstream implementation extension, not part of the OneBot 11
    /// standard API, and support depends on the upstream implementation. This method only
    /// sends a request and does not wait for a response. `Ok(())` only means the request
    /// was written to the current WebSocket connection; it does not mean the remark has
    /// been changed.
    ///
    /// # Arguments
    ///
    /// - `group_id`: target group number.
    /// - `remark`: new group remark, sent as-is. Whether an empty string clears the remark
    ///   is interpreted by the upstream implementation.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when no usable connection is available,
    /// and [`crate::error::Error::WebSocketError`] when the WebSocket write fails. If the
    /// upstream does not recognize the action or rejects the parameters, this method does
    /// not return the corresponding error.
    pub async fn set_group_remark(&self, group_id: i64, remark: &str) -> Result<()> {
        self.websocket_send(
            "set_group_remark",
            serde_json::json!({
                "group_id": group_id,
                "remark": remark,
            }),
        )
        .await?;
        Ok(())
    }
}
