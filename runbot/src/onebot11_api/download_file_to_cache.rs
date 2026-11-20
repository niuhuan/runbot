//! Downloads a file to the cache directory.
//!
//! The default version uses a thirty-second response wait; the download itself is performed by
//! the upstream.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use tokio::time::Duration;

/// Information about a file downloaded to the cache.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadedFile {
    /// Path of the cached file or the upstream file identifier.
    pub file: String,
}

impl BotContext {
    /// Requests that the upstream download a file to its cache directory and waits up to the
    /// given time for the response data.
    ///
    /// # Arguments
    ///
    /// - `url`: URL of the file to download.
    /// - `thread_count`: number of download threads; whether it is used depends on the upstream
    ///   implementation.
    /// - `headers`: JSON object of extra HTTP request headers; when `None`, the `headers` field
    ///   is not sent.
    /// - `timeout`: response wait duration after the request has been sent. The upstream may
    ///   only send the response after the download finishes, so a slow download consumes part
    ///   of this wait; local serialization and write time are not included.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when the current connection is unavailable
    /// or `retcode` is non-zero; returns [`crate::error::Error::WebSocketError`] when the
    /// WebSocket send fails; returns [`crate::error::Error::TimeoutError`] on timeout; returns
    /// [`crate::error::Error::JsonError`] when the response cannot be deserialized into
    /// [`DownloadedFile`].
    pub async fn download_file_to_cache_with_timeout(
        &self,
        url: &str,
        thread_count: i32,
        headers: Option<serde_json::Value>,
        timeout: Duration,
    ) -> Result<DownloadedFile> {
        let mut params = json!({
            "url": url,
            "thread_count": thread_count,
        });
        if let Some(h) = headers {
            params["headers"] = h;
        }
        let response = self
            .websocket_send("download_file_to_cache", params)
            .await?;
        let data = response.data(timeout).await?;
        let file: DownloadedFile = serde_json::from_value(data)?;
        Ok(file)
    }

    /// Calls [`Self::download_file_to_cache_with_timeout`] with a thirty-second response wait.
    ///
    /// See the timeout variant for other behavior and errors.
    pub async fn download_file_to_cache(
        &self,
        url: &str,
        thread_count: i32,
        headers: Option<serde_json::Value>,
    ) -> Result<DownloadedFile> {
        self.download_file_to_cache_with_timeout(url, thread_count, headers, Duration::from_secs(30))
            .await
    }
}

