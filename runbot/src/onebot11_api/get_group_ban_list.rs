use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use tokio::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupBanInfo {
    pub user_id: i64,
    pub nickname: String,
    pub time_left: i64,
}

impl BotContext {
    pub async fn get_group_ban_list_with_timeout(
        &self,
        group_id: i64,
        timeout: Duration,
    ) -> Result<Vec<GroupBanInfo>> {
        let response = self
            .websocket_send(
                "get_group_ban_list",
                json!({
                    "group_id": group_id,
                }),
            )
            .await?;
        let data = response.data(timeout).await?;
        let bans: Vec<GroupBanInfo> = serde_json::from_value(data)?;
        Ok(bans)
    }

    pub async fn get_group_ban_list(&self, group_id: i64) -> Result<Vec<GroupBanInfo>> {
        self.get_group_ban_list_with_timeout(group_id, Duration::from_secs(10))
            .await
    }
}

