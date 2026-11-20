//! Wrapper for the OneBot `set_group_avatar` action. Methods in this module only send
//! requests and do not wait for an upstream response.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_json::json;

impl BotContext {
    /// Requests that the avatar of the specified group be set to the specified file.
    ///
    /// This action is an upstream implementation extension, not part of the OneBot 11
    /// standard API, and support depends on the upstream implementation. This method only
    /// sends a request and does not wait for a response. `Ok(())` only means the request
    /// was written to the current WebSocket connection; it does not mean the group avatar
    /// has changed.
    ///
    /// # Arguments
    ///
    /// - `group_id`: target group number.
    /// - `file`: avatar file identifier, sent as-is. Acceptable formats are determined by
    ///   the upstream implementation.
    /// - `cache`: cache option, sent as-is in the `cache` field. Values and meaning are
    ///   defined by the upstream implementation; this library does not validate them.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when no usable connection is available,
    /// and [`crate::error::Error::WebSocketError`] when the WebSocket write fails. Rejected
    /// parameters and failed avatar changes are not returned by this method.
    pub async fn set_group_avatar(&self, group_id: i64, file: &str, cache: i32) -> Result<()> {
        self.websocket_send(
            "set_group_avatar",
            json!({
                "group_id": group_id,
                "file": file,
                "cache": cache,
            }),
        )
        .await?;
        Ok(())
    }
}

