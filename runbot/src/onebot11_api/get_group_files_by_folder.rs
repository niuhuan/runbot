use std::time::Duration;

use crate::error::Result;
use crate::prelude::BotContext;
use crate::prelude::get_group_root_files::GroupFilesData;
use serde_json::json;

impl BotContext {
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

    pub async fn get_group_files_by_folder(&self, group_id: i64) -> Result<GroupFilesData> {
        self.get_group_files_by_folder_with_timeout(group_id, Duration::from_secs(30))
            .await
    }
}
