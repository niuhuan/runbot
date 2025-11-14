use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use tokio::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendedGroup {
    pub group_id: i64,
    pub group_name: String,
    pub group_memo: String,
    pub group_create_time: i64,
    pub group_level: i64,
    pub member_count: i64,
    pub max_member_count: i64,
}

impl BotContext {
    pub async fn get_recommended_groups_with_timeout(
        &self,
        timeout: Duration,
    ) -> Result<Vec<RecommendedGroup>> {
        let response = self
            .websocket_send("get_recommended_groups", serde_json::Value::Null)
            .await?;
        let data = response.data(timeout).await?;
        let groups: Vec<RecommendedGroup> = serde_json::from_value(data)?;
        Ok(groups)
    }

    pub async fn get_recommended_groups(&self) -> Result<Vec<RecommendedGroup>> {
        self.get_recommended_groups_with_timeout(Duration::from_secs(10))
            .await
    }
}

