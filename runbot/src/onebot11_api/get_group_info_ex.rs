use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use tokio::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupInfoEx {
    pub group_id: i64,
    pub group_name: String,
    pub group_memo: String,
    pub group_create_time: i64,
    pub group_level: i64,
    pub member_count: i64,
    pub max_member_count: i64,
}

impl BotContext {
    pub async fn get_group_info_ex_with_timeout(
        &self,
        group_id: i64,
        no_cache: bool,
        timeout: Duration,
    ) -> Result<GroupInfoEx> {
        let response = self
            .websocket_send(
                "get_group_info_ex",
                json!({
                    "group_id": group_id,
                    "no_cache": no_cache,
                }),
            )
            .await?;
        let data = response.data(timeout).await?;
        let info: GroupInfoEx = serde_json::from_value(data)?;
        Ok(info)
    }

    pub async fn get_group_info_ex(&self, group_id: i64, no_cache: bool) -> Result<GroupInfoEx> {
        self.get_group_info_ex_with_timeout(group_id, no_cache, Duration::from_secs(10))
            .await
    }
}

