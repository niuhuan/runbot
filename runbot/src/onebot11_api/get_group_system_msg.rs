use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use tokio::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupSystemMsg {
    pub invited_requests: Vec<serde_json::Value>,
    pub join_requests: Vec<serde_json::Value>,
}

impl BotContext {
    pub async fn get_group_system_msg_with_timeout(
        &self,
        timeout: Duration,
    ) -> Result<GroupSystemMsg> {
        let response = self
            .websocket_send("get_group_system_msg", serde_json::Value::Null)
            .await?;
        let data = response.data(timeout).await?;
        let msg: GroupSystemMsg = serde_json::from_value(data)?;
        Ok(msg)
    }

    pub async fn get_group_system_msg(&self) -> Result<GroupSystemMsg> {
        self.get_group_system_msg_with_timeout(Duration::from_secs(10))
            .await
    }
}

