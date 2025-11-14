use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use tokio::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupHonorMember {
    pub user_id: i64,
    pub nickname: String,
    pub avatar: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupHonor {
    pub group_id: i64,
    pub current_talkative: Option<GroupHonorMember>,
    pub talkative_list: Vec<GroupHonorMember>,
    pub performer_list: Vec<GroupHonorMember>,
    pub legend_list: Vec<GroupHonorMember>,
    pub strong_newbie_list: Vec<GroupHonorMember>,
    pub emotion_list: Vec<GroupHonorMember>,
}

impl BotContext {
    pub async fn get_group_honor_with_timeout(
        &self,
        group_id: i64,
        honor_type: &str,
        timeout: Duration,
    ) -> Result<GroupHonor> {
        let response = self
            .websocket_send(
                "get_group_honor",
                json!({
                    "group_id": group_id,
                    "type": honor_type,
                }),
            )
            .await?;
        let data = response.data(timeout).await?;
        let honor: GroupHonor = serde_json::from_value(data)?;
        Ok(honor)
    }

    pub async fn get_group_honor(&self, group_id: i64, honor_type: &str) -> Result<GroupHonor> {
        self.get_group_honor_with_timeout(group_id, honor_type, Duration::from_secs(10))
            .await
    }
}

