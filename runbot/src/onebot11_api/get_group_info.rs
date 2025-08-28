use tokio::time::Duration;
use crate::prelude::BotContext;
use crate::error::Result;
use serde_derive::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupInfo {
    pub group_id: i64,
    pub group_name: String,
    pub member_count: i64,
    pub max_member_count: i64,
}

impl BotContext {
    pub async fn get_group_info(&self, group_id: i64, no_cache: bool) -> Result<GroupInfo> {
        let response = self.websocket_send("get_group_info", serde_json::json!({
            "group_id": group_id,
            "no_cache": no_cache,
        })).await?;
        let response = response.data(Duration::from_secs(10)).await?;
        let group_info: GroupInfo = serde_json::from_value(response)?;
        Ok(group_info)
    }
}

