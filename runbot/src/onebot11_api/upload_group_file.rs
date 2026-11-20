//! Upload a group file.
//!
//! The method returns an [`UploadGroupFileWating`] handle; the upload result is only
//! awaited and parsed after `wait` is called.

use std::time::Duration;

use crate::error::Result;
use crate::prelude::BotContext;
use crate::prelude::EchoAsyncResponse;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;

/// Data returned after a successful group file upload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadGroupFile {
    /// File ID after upload.
    pub file_id: String,
}

/// Asynchronous response handle for uploading a group file.
///
/// The type name preserves the spelling of the current API. The handle can be awaited only
/// once after it is consumed; dropping it does not cancel the upload request that has
/// already been sent.
pub struct UploadGroupFileWating(EchoAsyncResponse);

impl UploadGroupFileWating {
    /// Wait for the upload result within `timeout` and deserialize it into
    /// [`UploadGroupFile`].
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when the current connection is
    /// unavailable or `retcode` is non-zero; returns
    /// [`crate::error::Error::TimeoutError`] on timeout; returns
    /// [`crate::error::Error::JsonError`] when the response data cannot be deserialized.
    pub async fn wait(self, timeout: Duration) -> Result<UploadGroupFile> {
        let response = self.0.data(timeout).await?;
        let data: UploadGroupFile = serde_json::from_value(response)?;
        Ok(data)
    }
}

impl BotContext {
    /// Send a request to upload a group file and return a handle for waiting on the result.
    ///
    /// # Arguments
    ///
    /// - `group_id`: Target group number.
    /// - `file`: Local file path or a source accepted by the upstream; this method sends it
    ///   as-is.
    /// - `name`: File name after upload.
    /// - `folder_id`: Target folder ID; `None` is sent as JSON `null`, and whether that is
    ///   equivalent to the root directory depends on the upstream implementation.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when the current connection is
    /// unavailable; returns [`crate::error::Error::WebSocketError`] when the WebSocket send
    /// fails. The upload result must be obtained by waiting on the handle.
    pub async fn upload_group_file_with(
        &self,
        group_id: i64,
        file: &str,
        name: &str,
        folder_id: impl Into<Option<&str>>,
    ) -> Result<UploadGroupFileWating> {
        let response = self
            .websocket_send(
                "upload_group_file",
                json!({
                    "group_id": group_id,
                    "file": file,
                    "name": name,
                    "folder_id": folder_id.into(),
                }),
            )
            .await?;
        Ok(UploadGroupFileWating(response))
    }
}
