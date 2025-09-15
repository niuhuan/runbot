use std::time::Duration;

use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroupFilesData {
    pub files: Vec<GroupFile>,
    pub folders: Vec<GroupFolder>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroupFile {
    pub group_id: i64,
    pub file_id: String,
    pub file_name: String,
    pub busid: i64,
    pub size: i64,
    pub file_size: i64,
    pub upload_time: i64,
    pub dead_time: i64,
    pub modify_time: i64,
    pub download_times: i64,
    pub uploader: i64,
    pub uploader_name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroupFolder {
    pub group_id: i64,
    pub folder_id: String,
    pub folder: String,
    pub folder_name: String,
    pub create_time: i64,
    pub creator: i64,
    pub creator_name: String,
    pub total_file_count: i64,
}

impl BotContext {
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

    pub async fn get_group_root_files(&self, group_id: i64) -> Result<GroupFilesData> {
        self.get_group_root_files_with_timeout(group_id, Duration::from_secs(30))
            .await
    }
}
