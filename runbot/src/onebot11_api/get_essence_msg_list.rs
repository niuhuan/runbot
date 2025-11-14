use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use tokio::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EssenceMsg {
    pub sender_id: i64,
    pub sender_nick: String,
    pub sender_time: i64,
    pub operator_id: i64,
    pub operator_nick: String,
    pub operator_time: i64,
    pub message_id: i64,
}

impl BotContext {
    pub async fn get_essence_msg_list_with_timeout(
        &self,
        group_id: i64,
        timeout: Duration,
    ) -> Result<Vec<EssenceMsg>> {
        let response = self
            .websocket_send(
                "get_essence_msg_list",
                json!({
                    "group_id": group_id,
                }),
            )
            .await?;
        let data = response.data(timeout).await?;
        let messages: Vec<EssenceMsg> = serde_json::from_value(data)?;
        Ok(messages)
    }

    pub async fn get_essence_msg_list(&self, group_id: i64) -> Result<Vec<EssenceMsg>> {
        self.get_essence_msg_list_with_timeout(group_id, Duration::from_secs(10))
            .await
    }
}

