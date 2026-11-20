//! Create a group file folder.
//!
//! The method returns a [`CreateGroupFileFolderWating`] handle; the upstream creation result
//! is only awaited and parsed after its `wait` method is called. The result fields contain a
//! nested structure extended by the upstream.

use crate::error::Result;
use crate::prelude::BotContext;
use crate::prelude::EchoAsyncResponse;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use std::time::Duration;

/// Response data after creating a group folder.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateGroupFileFolderData {
    /// Upstream return code and description.
    pub result: CreateGroupFileFolderResult,
    /// Group and folder information; the JSON field name is `groupItem`.
    #[serde(rename = "groupItem")]
    pub group_item: CreateGroupFileFolderGroupItem,
}

/// Upstream result code and description for creating a group folder.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateGroupFileFolderResult {
    /// Upstream business return code; the JSON field name is `retCode`.
    #[serde(rename = "retCode")]
    pub ret_code: i64,
    /// Upstream return description; the JSON field name is `retMsg`.
    #[serde(rename = "retMsg")]
    pub ret_msg: String,
    /// Client-facing prompt text; the JSON field name is `clientWording`.
    #[serde(rename = "clientWording")]
    pub client_wording: String,
}

/// Group and folder information for the newly created folder.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateGroupFileFolderGroupItem {
    /// Peer identifier; the JSON field name is `peerId`.
    #[serde(rename = "peerId")]
    pub peer_id: String,
    /// Entry type; the JSON field name is `type`, meaning determined by the upstream.
    #[serde(rename = "type")]
    pub type_field: i64,
    /// Folder information; the JSON field name is `folderInfo`.
    #[serde(rename = "folderInfo")]
    pub folder_info: FolderInfo,
    /// File information; the JSON field name is `fileInfo`. Uses an empty string when
    /// missing, and JSON `null` is also converted to an empty string (`null_to_default`).
    #[serde(
        rename = "fileInfo",
        default,
        deserialize_with = "crate::common::null_to_default"
    )]
    pub file_info: String,
}

/// Detailed group folder information.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FolderInfo {
    /// Folder ID; the JSON field name is `folderId`.
    #[serde(rename = "folderId")]
    pub folder_id: String,
    /// Parent folder ID; the JSON field name is `parentFolderId`.
    #[serde(rename = "parentFolderId")]
    pub parent_folder_id: String,
    /// Folder name; the JSON field name is `folderName`.
    #[serde(rename = "folderName")]
    pub folder_name: String,
    /// Creation time; the JSON field name is `createTime`, unit defined by the upstream.
    #[serde(rename = "createTime")]
    pub create_time: i64,
    /// Modification time; the JSON field name is `modifyTime`, unit defined by the upstream.
    #[serde(rename = "modifyTime")]
    pub modify_time: i64,
    /// Creator account identifier; the JSON field name is `createUin`.
    #[serde(rename = "createUin")]
    pub create_uin: String,
    /// Creator display name; the JSON field name is `creatorName`.
    #[serde(rename = "creatorName")]
    pub creator_name: String,
    /// Total number of files in the folder; the JSON field name is `totalFileCount`.
    #[serde(rename = "totalFileCount")]
    pub total_file_count: i64,
    /// Modifier account identifier; the JSON field name is `modifyUin`.
    #[serde(rename = "modifyUin")]
    pub modify_uin: String,
    /// Modifier display name; the JSON field name is `modifyName`.
    #[serde(rename = "modifyName")]
    pub modify_name: String,
    /// Used space; the JSON field name is `usedSpace`, unit defined by the upstream.
    #[serde(rename = "usedSpace")]
    pub used_space: String,
}

/// Asynchronous response handle for creating a group folder.
///
/// The type name preserves the spelling of the current API. The handle can be awaited only
/// once after it is consumed; dropping it does not cancel the request that has already been
/// sent.
pub struct CreateGroupFileFolderWating(EchoAsyncResponse);

impl CreateGroupFileFolderWating {
    /// Wait for the creation result within `timeout` and deserialize it into
    /// [`CreateGroupFileFolderData`].
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when the current connection is
    /// unavailable or `retcode` is non-zero; returns
    /// [`crate::error::Error::TimeoutError`] on timeout; returns
    /// [`crate::error::Error::JsonError`] when the response data cannot be deserialized.
    pub async fn wait(self, timeout: Duration) -> Result<CreateGroupFileFolderData> {
        let response = self.0.data(timeout).await?;
        let data: CreateGroupFileFolderData = serde_json::from_value(response)?;
        Ok(data)
    }
}

impl BotContext {
    /// Send a request to create a group folder and return a handle for waiting on the result.
    ///
    /// An `Ok` return only means the request has been written to the current connection; call
    /// [`CreateGroupFileFolderWating::wait`] to obtain the upstream result.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when the current connection is
    /// unavailable; returns [`crate::error::Error::WebSocketError`] when the WebSocket send
    /// fails.
    pub async fn create_group_file_folder(
        &self,
        group_id: i64,
        folder_name: &str,
    ) -> Result<CreateGroupFileFolderWating> {
        let response = self
            .websocket_send(
                "create_group_file_folder",
                json!({
                    "group_id": group_id,
                    "folder_name": folder_name,
                }),
            )
            .await?;
        Ok(CreateGroupFileFolderWating(response))
    }
}
