//! Kicks multiple group members (upstream extension action `batch_kick_group_member`).
//!
//! This module only sends the request and does not wait for a OneBot response.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_json::json;

impl BotContext {
    /// Sends a request to kick multiple group members.
    ///
    /// This method does not wait for a OneBot response. Returning `Ok(())` only means the
    /// request was written to the current WebSocket connection; it does not mean the members
    /// have been kicked.
    ///
    /// # Arguments
    ///
    /// - `group_id`: target group ID.
    /// - `user_ids`: QQ IDs of the members to kick. This method does not validate the count,
    ///   whether the members belong to the group, or whether the bot itself is included.
    /// - `reject_add_request`: whether to also reject these members' future join requests.
    ///   The exact behavior is determined by the upstream implementation.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when no connection is available, or
    /// [`crate::error::Error::WebSocketError`] when the WebSocket write fails. Upstream
    /// permission, member-list, or parameter validation failures are not waited for or
    /// returned by this method.
    pub async fn batch_kick_group_member(
        &self,
        group_id: i64,
        user_ids: Vec<i64>,
        reject_add_request: bool,
    ) -> Result<()> {
        self.websocket_send(
            "batch_kick_group_member",
            json!({
                "group_id": group_id,
                "user_ids": user_ids,
                "reject_add_request": reject_add_request,
            }),
        )
        .await?;
        Ok(())
    }
}

