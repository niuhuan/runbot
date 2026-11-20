//! Wraps the OneBot action `move_group_file`.
//!
//! The method writes the parameters for moving a group file to the current
//! WebSocket connection and returns without waiting for the upstream's business
//! response.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_json::json;

impl BotContext {
    /// Requests that a group file be moved to the specified folder.
    ///
    /// `Ok(())` means only that the request has been written to the current
    /// WebSocket; it does not mean the file has moved. Whether the folder exists
    /// and whether the account has permission are not awaited.
    ///
    /// # Arguments
    ///
    /// - `group_id`: number of the group that owns the file.
    /// - `file_id`: ID of the group file to move, returned by a group file
    ///   listing API.
    /// - `folder_id`: ID of the target folder; this library does not check that
    ///   it exists and does not special-case an empty string.
    ///
    /// # Notes
    ///
    /// This action depends on the upstream implementation, and implementations
    /// may use different parameter formats for file and folder IDs; this library
    /// sends the field names above unchanged.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when the current connection is
    /// unavailable (not connected, disconnected, or closed);
    /// [`crate::error::Error::WebSocketError`] when the WebSocket write fails.
    /// Upstream path validation or permission errors are not awaited or returned
    /// by this method.
    pub async fn move_group_file(
        &self,
        group_id: i64,
        file_id: &str,
        folder_id: &str,
    ) -> Result<()> {
        self.websocket_send(
            "move_group_file",
            json!({
                "group_id": group_id,
                "file_id": file_id,
                "folder_id": folder_id,
            }),
        )
        .await?;
        Ok(())
    }
}

