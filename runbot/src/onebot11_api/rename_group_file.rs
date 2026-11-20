//! Wraps the OneBot action `rename_group_file`.
//!
//! The method writes the parameters for renaming a group file to the current
//! WebSocket connection and returns without waiting for the upstream's business
//! response.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_json::json;

impl BotContext {
    /// Requests that a group file be renamed.
    ///
    /// `Ok(())` means only that the request has been written to the current
    /// WebSocket; it does not mean the file name has changed. Permission and
    /// duplicate-name checks are not awaited.
    ///
    /// # Arguments
    ///
    /// - `group_id`: number of the group that owns the file.
    /// - `file_id`: ID of the group file to rename, returned by a group file
    ///   listing API.
    /// - `name`: new file name; this library does not validate its length or
    ///   check for invalid characters.
    ///
    /// # Notes
    ///
    /// This action depends on the upstream implementation, and implementations
    /// may use different parameter formats for the file ID and name; this
    /// library sends the field names above unchanged.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when the current connection is
    /// unavailable (not connected, disconnected, or closed);
    /// [`crate::error::Error::WebSocketError`] when the WebSocket write fails.
    /// Upstream permission or file name validation errors are not awaited or
    /// returned by this method.
    pub async fn rename_group_file(
        &self,
        group_id: i64,
        file_id: &str,
        name: &str,
    ) -> Result<()> {
        self.websocket_send(
            "rename_group_file",
            json!({
                "group_id": group_id,
                "file_id": file_id,
                "name": name,
            }),
        )
        .await?;
        Ok(())
    }
}

