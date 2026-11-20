//! Wraps the OneBot action `rename_group_file_folder`.
//!
//! The method sends a request to rename a group file folder and returns an
//! [`EchoAsyncResponse`] handle that the caller must use to wait for the upstream
//! response.

use crate::error::Result;
use crate::prelude::BotContext;
use crate::prelude::EchoAsyncResponse;
use serde_json::json;

impl BotContext {
    /// Requests that a group file folder be renamed and returns a handle for
    /// waiting on the response.
    ///
    /// `Ok` means only that the request has been written to the current
    /// WebSocket; it does not mean the folder has been renamed. The caller must
    /// continue waiting on the handle for the business result.
    ///
    /// # Arguments
    ///
    /// - `group_id`: number of the group that owns the folder.
    /// - `folder_id`: ID of the folder to rename, returned by a group file
    ///   listing API.
    /// - `new_folder_name`: new folder name; this library does not validate its
    ///   length or check for invalid characters.
    ///
    /// # Returns
    ///
    /// Returns an [`EchoAsyncResponse`] handle. Call
    /// [`EchoAsyncResponse::response`], [`EchoAsyncResponse::data`], or
    /// [`EchoAsyncResponse::ok`] to wait for the result; these methods consume the
    /// handle, so it can be awaited only once.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when the current connection is
    /// unavailable (not connected, disconnected, or closed);
    /// [`crate::error::Error::WebSocketError`] when the WebSocket write fails.
    /// This stage does not check `retcode` and does not return
    /// [`crate::error::Error::TimeoutError`].
    ///
    /// # Cancellation
    ///
    /// Dropping the handle removes the local echo registration but does not
    /// withdraw the request that has already been sent; the handle can no longer
    /// be used to obtain the response.
    pub async fn rename_group_file_folder(
        &self,
        group_id: i64,
        folder_id: &str,
        new_folder_name: &str,
    ) -> Result<EchoAsyncResponse> {
        self.websocket_send(
            "rename_group_file_folder",
            json!({
                "group_id": group_id,
                "folder_id": folder_id,
                "new_folder_name": new_folder_name,
            }),
        )
        .await
    }
}
