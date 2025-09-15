use crate::error::Result;
use crate::prelude::BotContext;
use crate::prelude::EchoAsyncResponse;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use std::time::Duration;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateGroupFileFolderData {
    pub result: CreateGroupFileFolderResult,
    #[serde(rename = "groupItem")]
    pub group_item: CreateGroupFileFolderGroupItem,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateGroupFileFolderResult {
    #[serde(rename = "retCode")]
    pub ret_code: i64,
    #[serde(rename = "retMsg")]
    pub ret_msg: String,
    #[serde(rename = "clientWording")]
    pub client_wording: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateGroupFileFolderGroupItem {
    #[serde(rename = "peerId")]
    pub peer_id: String,
    #[serde(rename = "type")]
    pub type_field: i64,
    #[serde(rename = "folderInfo")]
    pub folder_info: FolderInfo,
    #[serde(
        rename = "fileInfo",
        default,
        deserialize_with = "crate::common::null_to_default"
    )]
    pub file_info: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FolderInfo {
    #[serde(rename = "folderId")]
    pub folder_id: String,
    #[serde(rename = "parentFolderId")]
    pub parent_folder_id: String,
    #[serde(rename = "folderName")]
    pub folder_name: String,
    #[serde(rename = "createTime")]
    pub create_time: i64,
    #[serde(rename = "modifyTime")]
    pub modify_time: i64,
    #[serde(rename = "createUin")]
    pub create_uin: String,
    #[serde(rename = "creatorName")]
    pub creator_name: String,
    #[serde(rename = "totalFileCount")]
    pub total_file_count: i64,
    #[serde(rename = "modifyUin")]
    pub modify_uin: String,
    #[serde(rename = "modifyName")]
    pub modify_name: String,
    #[serde(rename = "usedSpace")]
    pub used_space: String,
}

pub struct CreateGroupFileFolderWating(EchoAsyncResponse);

impl CreateGroupFileFolderWating {
    pub async fn wait(self, timeout: Duration) -> Result<CreateGroupFileFolderData> {
        let response = self.0.data(timeout).await?;
        let data: CreateGroupFileFolderData = serde_json::from_value(response)?;
        Ok(data)
    }
}

impl BotContext {
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
