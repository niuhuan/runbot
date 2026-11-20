//! Deletes a group folder (OneBot 11 action `delete_group_folder`).
//!
//! The method sends the request and returns a response handle; a wait method on the handle
//! must be called to confirm the upstream result.

use crate::error::Result;
use crate::prelude::BotContext;
use crate::prelude::EchoAsyncResponse;
use serde_json::json;

impl BotContext {
    /// Sends a request to delete the specified group folder and returns a handle for waiting
    /// on the response.
    ///
    /// Returning `Ok` only means the request was written to the current WebSocket; it does
    /// not mean the folder has been deleted. The response `data` for this action is usually
    /// empty; call [`crate::bot_context::EchoAsyncResponse::ok`] to wait for the response and
    /// check `retcode`.
    ///
    /// # Arguments
    ///
    /// - `group_id`: ID of the group containing the folder.
    /// - `folder_id`: ID of the group folder to delete, usually obtained from a group file
    ///   system API. This method does not validate its format.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when no connection is available, or
    /// [`crate::error::Error::WebSocketError`] when the WebSocket write fails. Business
    /// errors such as insufficient upstream permissions, a missing folder, or a non-zero
    /// `retcode` appear when waiting on the handle; this method does not wait.
    ///
    /// # Cancellation
    ///
    /// Dropping the returned handle removes the local echo registration but does not revoke
    /// the delete request already sent. A timeout or cancellation does not prove the
    /// upstream did not execute it.
    pub async fn delete_group_folder(
        &self,
        group_id: i64,
        folder_id: &str,
    ) -> Result<EchoAsyncResponse> {
        self.websocket_send(
            "delete_group_folder",
            json!({
                "group_id": group_id,
                "folder_id": folder_id,
            }),
        )
        .await
    }
}
