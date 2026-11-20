//! Deletes a group notice (upstream extension action `delete_group_notice`).
//!
//! This module only sends the request and does not wait for a OneBot response.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_json::json;

impl BotContext {
    /// Sends a request to delete the specified group notice.
    ///
    /// This method does not wait for a OneBot response. Returning `Ok(())` only means the
    /// request was written to the current WebSocket connection; it does not mean the notice
    /// has been deleted.
    ///
    /// # Arguments
    ///
    /// - `group_id`: ID of the group containing the notice.
    /// - `notice_id`: ID of the group notice to delete, usually obtained from a group notice
    ///   listing API. This method does not validate its format.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when no connection is available, or
    /// [`crate::error::Error::WebSocketError`] when the WebSocket write fails. Business
    /// errors such as insufficient caller permissions, a missing notice, or upstream
    /// rejection are not waited for or returned by this method.
    pub async fn delete_group_notice(&self, group_id: i64, notice_id: &str) -> Result<()> {
        self.websocket_send(
            "delete_group_notice",
            json!({
                "group_id": group_id,
                "notice_id": notice_id,
            }),
        )
        .await?;
        Ok(())
    }
}

