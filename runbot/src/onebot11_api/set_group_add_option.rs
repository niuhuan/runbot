//! Wrapper for the OneBot `set_group_add_option` action. Methods in this module only send
//! requests and do not wait for an upstream response.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_json::json;

impl BotContext {
    /// Requests that the specified group's join options be set.
    ///
    /// This action is an upstream implementation extension, not part of the OneBot 11
    /// standard API; support and parameter values depend on the upstream implementation.
    /// This method only sends a request and does not wait for a response. `Ok(())` only
    /// means the request was written to the current WebSocket connection; it does not mean
    /// the group options have been changed.
    ///
    /// # Arguments
    ///
    /// - `group_id`: target group number.
    /// - `sub_type`: subtype string for the join option, sent as-is. Values are defined by
    ///   the upstream implementation; this library does not validate them.
    /// - `approve`: boolean option that allows or approves the setting identified by
    ///   `sub_type`. The exact effect is interpreted by the upstream implementation.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when no usable connection is available,
    /// and [`crate::error::Error::WebSocketError`] when the WebSocket write fails. If the
    /// upstream does not recognize the action or rejects the parameters, this method does
    /// not return the corresponding error.
    pub async fn set_group_add_option(&self, group_id: i64, sub_type: &str, approve: bool) -> Result<()> {
        self.websocket_send(
            "set_group_add_option",
            json!({
                "group_id": group_id,
                "sub_type": sub_type,
                "approve": approve,
            }),
        )
        .await?;
        Ok(())
    }
}

