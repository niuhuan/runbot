use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use tokio::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStatus {
    pub user_id: i64,
    pub status: i32,
}

impl BotContext {
    pub async fn get_user_status_with_timeout(
        &self,
        user_id: i64,
        timeout: Duration,
    ) -> Result<UserStatus> {
        let response = self
            .websocket_send(
                "get_user_status",
                json!({
                    "user_id": user_id,
                }),
            )
            .await?;
        let data = response.data(timeout).await?;
        let status: UserStatus = serde_json::from_value(data)?;
        Ok(status)
    }

    pub async fn get_user_status(&self, user_id: i64) -> Result<UserStatus> {
        self.get_user_status_with_timeout(user_id, Duration::from_secs(10))
            .await
    }
}

