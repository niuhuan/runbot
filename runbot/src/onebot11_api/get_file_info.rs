//! Get file information.
//!
//! The default version waits up to ten seconds for the response; returns the size, file
//! name, and file ID.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use tokio::time::Duration;

/// File information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    /// File size; the unit is defined by the upstream (usually bytes).
    pub size: i64,
    /// File name.
    pub filename: String,
    /// File ID.
    pub file_id: String,
}

impl BotContext {
    /// Request file information and wait for the response data within the specified
    /// timeout.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when the current connection is
    /// unavailable or `retcode` is non-zero; returns
    /// [`crate::error::Error::WebSocketError`] when the WebSocket send fails; returns
    /// [`crate::error::Error::TimeoutError`] on timeout; returns
    /// [`crate::error::Error::JsonError`] when the response cannot be deserialized into
    /// [`FileInfo`].
    pub async fn get_file_info_with_timeout(
        &self,
        file_id: &str,
        timeout: Duration,
    ) -> Result<FileInfo> {
        let response = self
            .websocket_send(
                "get_file_info",
                json!({
                    "file_id": file_id,
                }),
            )
            .await?;
        let data = response.data(timeout).await?;
        let info: FileInfo = serde_json::from_value(data)?;
        Ok(info)
    }

    /// Call [`Self::get_file_info_with_timeout`] with a ten-second response wait.
    ///
    /// See the timeout version for other behavior and errors.
    pub async fn get_file_info(&self, file_id: &str) -> Result<FileInfo> {
        self.get_file_info_with_timeout(file_id, Duration::from_secs(10))
            .await
    }
}

