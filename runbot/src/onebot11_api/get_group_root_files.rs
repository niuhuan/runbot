//! Fetches the root directory listing of group files.
//!
//! The default version uses a thirty-second response wait; most group file and folder fields
//! are upstream extension information.

use std::time::Duration;

use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;

/// Contents of a group file directory.
///
/// Files and folders are stored in two separate lists; the order is determined by the
/// upstream response.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroupFilesData {
    /// Files in the current directory.
    pub files: Vec<GroupFile>,
    /// Subfolders in the current directory.
    pub folders: Vec<GroupFolder>,
}

/// A group file entry.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroupFile {
    /// Group ID that owns the file.
    pub group_id: i64,
    /// File ID, usable with file APIs such as download, move, and rename.
    pub file_id: String,
    /// File name.
    pub file_name: String,
    /// File business ID used by the upstream.
    pub busid: i64,
    /// File size; the unit is defined by the upstream (usually bytes).
    pub size: i64,
    /// Extended form of the file size field; the unit is defined by the upstream and may
    /// duplicate [`Self::size`].
    pub file_size: i64,
    /// Upload time, Unix timestamp in seconds.
    pub upload_time: i64,
    /// Expiration time, Unix timestamp in seconds; the meaning of `0` is determined by the
    /// upstream.
    pub dead_time: i64,
    /// Last modification time, Unix timestamp in seconds.
    pub modify_time: i64,
    /// Download count.
    pub download_times: i64,
    /// QQ ID of the uploader.
    pub uploader: i64,
    /// Display name of the uploader.
    pub uploader_name: String,
}

/// A group folder entry.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroupFolder {
    /// Group ID that owns the folder.
    pub group_id: i64,
    /// Folder ID, usable with APIs such as fetching files by folder.
    pub folder_id: String,
    /// Folder identifier; the exact meaning is determined by the upstream.
    pub folder: String,
    /// Folder name.
    pub folder_name: String,
    /// Creation time, Unix timestamp in seconds.
    pub create_time: i64,
    /// QQ ID of the creator.
    pub creator: i64,
    /// Display name of the creator.
    pub creator_name: String,
    /// Total number of files in the folder.
    pub total_file_count: i64,
}

impl BotContext {
    /// Requests the group file root directory and waits up to the given time for the response
    /// data.
    ///
    /// `timeout` only limits the response wait after the request has been sent; it does not
    /// include the send phase.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when the current connection is unavailable
    /// or `retcode` is non-zero; returns [`crate::error::Error::WebSocketError`] when the
    /// WebSocket send fails; returns [`crate::error::Error::TimeoutError`] on timeout; returns
    /// [`crate::error::Error::JsonError`] when the response cannot be deserialized into
    /// [`GroupFilesData`].
    pub async fn get_group_root_files_with_timeout(
        &self,
        group_id: i64,
        timeout: std::time::Duration,
    ) -> Result<GroupFilesData> {
        let response = self
            .websocket_send(
                "get_group_root_files",
                json!({
                    "group_id": group_id,
                }),
            )
            .await?;
        let response = response.data(timeout).await?;
        let data: GroupFilesData = serde_json::from_value(response)?;
        Ok(data)
    }

    /// Calls [`Self::get_group_root_files_with_timeout`] with a thirty-second response wait.
    ///
    /// See the timeout variant for other behavior and errors.
    pub async fn get_group_root_files(&self, group_id: i64) -> Result<GroupFilesData> {
        self.get_group_root_files_with_timeout(group_id, Duration::from_secs(30))
            .await
    }
}
