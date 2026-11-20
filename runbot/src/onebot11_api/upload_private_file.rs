//! Upload a private file.
//!
//! The method returns an [`UploadPrivateFileWaiting`] handle; the upload result is only
//! awaited and parsed after `wait` is called.

use crate::error::Result;
use crate::prelude::BotContext;
use crate::prelude::EchoAsyncResponse;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use std::time::Duration;

/// Data returned after a successful private file upload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadPrivateFile {
    /// File ID after upload.
    pub file_id: String,
}

/// Asynchronous response handle for uploading a private file.
///
/// The handle can be awaited only once after it is consumed; dropping it does not cancel the
/// upload request that has already been sent.
pub struct UploadPrivateFileWaiting(EchoAsyncResponse);

impl UploadPrivateFileWaiting {
    /// Wait for the upload result within `timeout` and deserialize it into
    /// [`UploadPrivateFile`].
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when the current connection is
    /// unavailable or `retcode` is non-zero; returns
    /// [`crate::error::Error::TimeoutError`] on timeout; returns
    /// [`crate::error::Error::JsonError`] when the response data cannot be deserialized.
    pub async fn wait(self, timeout: Duration) -> Result<UploadPrivateFile> {
        let response = self.0.data(timeout).await?;
        let data: UploadPrivateFile = serde_json::from_value(response)?;
        Ok(data)
    }
}

impl BotContext {
    /// Send a request to upload a private file and return a handle for waiting on the
    /// result.
    ///
    /// # Arguments
    ///
    /// - `user_id`: QQ number of the recipient.
    /// - `file`: Local file path or a source accepted by the upstream; this method sends it
    ///   as-is.
    /// - `name`: File name after upload.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when the current connection is
    /// unavailable; returns [`crate::error::Error::WebSocketError`] when the WebSocket send
    /// fails. The upload result must be obtained by waiting on the handle.
    pub async fn upload_private_file(
        &self,
        user_id: i64,
        file: &str,
        name: &str,
    ) -> Result<UploadPrivateFileWaiting> {
        let response = self
            .websocket_send(
                "upload_private_file",
                json!({
                    "user_id": user_id,
                    "file": file,
                    "name": name,
                }),
            )
            .await?;
        Ok(UploadPrivateFileWaiting(response))
    }
}
