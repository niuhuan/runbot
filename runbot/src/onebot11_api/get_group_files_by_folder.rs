//! Get files in a group folder.
//!
//! Note: the current implementation sends only `group_id` and does not send a folder ID;
//! it therefore cannot select a specific folder, and the directory scope actually returned
//! by the upstream depends on its default behavior.

use std::time::Duration;

use crate::error::Result;
use crate::prelude::BotContext;
use crate::prelude::get_group_root_files::GroupFilesData;
use serde_json::json;

impl BotContext {
    /// Request the contents of a group folder and wait for the response data within the
    /// specified timeout.
    ///
    /// The current request body contains only `group_id`, with no `folder_id` parameter;
    /// this method cannot select a specific folder, and the returned contents are determined
    /// by how the upstream handles the request. The suffix-free method of the same name uses
    /// a thirty-second response wait.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when the current connection is
    /// unavailable or `retcode` is non-zero; returns
    /// [`crate::error::Error::WebSocketError`] when the WebSocket send fails; returns
    /// [`crate::error::Error::TimeoutError`] on timeout; returns
    /// [`crate::error::Error::JsonError`] when the response cannot be deserialized into
    /// [`GroupFilesData`].
    pub async fn get_group_files_by_folder_with_timeout(
        &self,
        group_id: i64,
        timeout: std::time::Duration,
    ) -> Result<GroupFilesData> {
        let response = self
            .websocket_send(
                "get_group_files_by_folder",
                json!({
                    "group_id": group_id,
                }),
            )
            .await?;
        let response = response.data(timeout).await?;
        let data: GroupFilesData = serde_json::from_value(response)?;
        Ok(data)
    }

    /// Call [`Self::get_group_files_by_folder_with_timeout`] with a thirty-second response
    /// wait.
    ///
    /// See the timeout version for other behavior and limitations.
    pub async fn get_group_files_by_folder(&self, group_id: i64) -> Result<GroupFilesData> {
        self.get_group_files_by_folder_with_timeout(group_id, Duration::from_secs(30))
            .await
    }
}
