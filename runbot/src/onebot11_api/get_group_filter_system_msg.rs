use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use tokio::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupFilterSystemMsg {
    pub group_id: i64,
    pub filter_type: Vec<String>,
}

impl BotContext {
    pub async fn get_group_filter_system_msg_with_timeout(
        &self,
        group_id: i64,
        timeout: Duration,
    ) -> Result<GroupFilterSystemMsg> {
        let response = self
            .websocket_send(
                "get_group_filter_system_msg",
                json!({
                    "group_id": group_id,
                }),
            )
            .await?;
        let data = response.data(timeout).await?;
        let msg: GroupFilterSystemMsg = serde_json::from_value(data)?;
        Ok(msg)
    }

    pub async fn get_group_filter_system_msg(&self, group_id: i64) -> Result<GroupFilterSystemMsg> {
        self.get_group_filter_system_msg_with_timeout(group_id, Duration::from_secs(10))
            .await
    }
}

