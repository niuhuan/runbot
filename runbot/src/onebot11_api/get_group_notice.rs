use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use tokio::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupNotice {
    pub sender_id: i64,
    pub publish_time: i64,
    pub message: serde_json::Value,
}

impl BotContext {
    pub async fn get_group_notice_with_timeout(
        &self,
        group_id: i64,
        timeout: Duration,
    ) -> Result<Vec<GroupNotice>> {
        let response = self
            .websocket_send(
                "get_group_notice",
                json!({
                    "group_id": group_id,
                }),
            )
            .await?;
        let data = response.data(timeout).await?;
        let notices: Vec<GroupNotice> = serde_json::from_value(data)?;
        Ok(notices)
    }

    pub async fn get_group_notice(&self, group_id: i64) -> Result<Vec<GroupNotice>> {
        self.get_group_notice_with_timeout(group_id, Duration::from_secs(10))
            .await
    }
}

