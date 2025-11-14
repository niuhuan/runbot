use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use tokio::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupAtAllRemain {
    pub can_at_all: bool,
    pub remain_at_all_count_for_group: i32,
    pub remain_at_all_count_for_uin: i32,
}

impl BotContext {
    pub async fn get_group_at_all_remain_with_timeout(
        &self,
        group_id: i64,
        timeout: Duration,
    ) -> Result<GroupAtAllRemain> {
        let response = self
            .websocket_send(
                "get_group_at_all_remain",
                json!({
                    "group_id": group_id,
                }),
            )
            .await?;
        let data = response.data(timeout).await?;
        let remain: GroupAtAllRemain = serde_json::from_value(data)?;
        Ok(remain)
    }

    pub async fn get_group_at_all_remain(&self, group_id: i64) -> Result<GroupAtAllRemain> {
        self.get_group_at_all_remain_with_timeout(group_id, Duration::from_secs(10))
            .await
    }
}

