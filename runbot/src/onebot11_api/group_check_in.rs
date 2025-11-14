use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use tokio::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupCheckIn {
    pub result: i32,
    pub point: i32,
}

impl BotContext {
    pub async fn group_check_in_with_timeout(
        &self,
        group_id: i64,
        timeout: Duration,
    ) -> Result<GroupCheckIn> {
        let response = self
            .websocket_send(
                "group_check_in",
                json!({
                    "group_id": group_id,
                }),
            )
            .await?;
        let data = response.data(timeout).await?;
        let check_in: GroupCheckIn = serde_json::from_value(data)?;
        Ok(check_in)
    }

    pub async fn group_check_in(&self, group_id: i64) -> Result<GroupCheckIn> {
        self.group_check_in_with_timeout(group_id, Duration::from_secs(10))
            .await
    }
}

