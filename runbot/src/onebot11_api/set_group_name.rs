//! Wrapper for the OneBot `set_group_name` action. Methods in this module only send
//! requests and do not wait for an upstream response.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_json::json;

impl BotContext {
    /// Requests that the name of the specified group be changed.
    ///
    /// This method only sends a `set_group_name` request and does not wait for an upstream
    /// response. `Ok(())` only means the request was written to the current WebSocket
    /// connection; it does not mean the group name has been changed.
    ///
    /// # Arguments
    ///
    /// - `group_id`: target group number.
    /// - `group_name`: new group name, sent as-is. Length, character, and permission
    ///   restrictions are validated by the upstream implementation.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when no usable connection is available,
    /// and [`crate::error::Error::WebSocketError`] when the WebSocket write fails. Upstream
    /// business errors such as insufficient permissions are not returned by this method.
    pub async fn set_group_name(&self, group_id: i64, group_name: &str) -> Result<()> {
        let msg = json!(
            {
                "group_id": group_id,
                "group_name": group_name,
            }
        );
        self.websocket_send("set_group_name", msg).await?;
        Ok(())
    }
}
