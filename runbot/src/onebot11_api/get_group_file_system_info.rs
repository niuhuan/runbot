use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use tokio::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupFileSystemInfo {
    pub file_count: i64,
    pub limit_count: i64,
    pub used_space: i64,
    pub total_space: i64,
}

impl BotContext {
    pub async fn get_group_file_system_info_with_timeout(
        &self,
        group_id: i64,
        timeout: Duration,
    ) -> Result<GroupFileSystemInfo> {
        let response = self
            .websocket_send(
                "get_group_file_system_info",
                json!({
                    "group_id": group_id,
                }),
            )
            .await?;
        let data = response.data(timeout).await?;
        let info: GroupFileSystemInfo = serde_json::from_value(data)?;
        Ok(info)
    }

    pub async fn get_group_file_system_info(&self, group_id: i64) -> Result<GroupFileSystemInfo> {
        self.get_group_file_system_info_with_timeout(group_id, Duration::from_secs(10))
            .await
    }
}

