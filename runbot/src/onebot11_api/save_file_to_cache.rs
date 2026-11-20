//! Save a file to the cache directory.
//!
//! The default version waits up to ten seconds for the response; the save operation is
//! performed by the upstream.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use tokio::time::Duration;

/// File information after saving.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedFile {
    /// Path of the cached file or an upstream file identifier.
    pub file: String,
}

impl BotContext {
    /// Request that the file be saved to the cache directory and wait for the response data
    /// within the specified timeout.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when the current connection is
    /// unavailable or `retcode` is non-zero; returns
    /// [`crate::error::Error::WebSocketError`] when the WebSocket send fails; returns
    /// [`crate::error::Error::TimeoutError`] on timeout; returns
    /// [`crate::error::Error::JsonError`] when the response cannot be deserialized into
    /// [`SavedFile`].
    pub async fn save_file_to_cache_with_timeout(
        &self,
        file_id: &str,
        timeout: Duration,
    ) -> Result<SavedFile> {
        let response = self
            .websocket_send(
                "save_file_to_cache",
                json!({
                    "file_id": file_id,
                }),
            )
            .await?;
        let data = response.data(timeout).await?;
        let saved: SavedFile = serde_json::from_value(data)?;
        Ok(saved)
    }

    /// Call [`Self::save_file_to_cache_with_timeout`] with a ten-second response wait.
    ///
    /// See the timeout version for other behavior and errors.
    pub async fn save_file_to_cache(&self, file_id: &str) -> Result<SavedFile> {
        self.save_file_to_cache_with_timeout(file_id, Duration::from_secs(10))
            .await
    }
}

