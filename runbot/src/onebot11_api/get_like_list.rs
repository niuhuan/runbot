use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use tokio::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LikeInfo {
    pub user_id: i64,
    pub nickname: String,
    pub time: i64,
}

impl BotContext {
    pub async fn get_like_list_with_timeout(
        &self,
        timeout: Duration,
    ) -> Result<Vec<LikeInfo>> {
        let response = self
            .websocket_send("get_like_list", serde_json::Value::Null)
            .await?;
        let data = response.data(timeout).await?;
        let likes: Vec<LikeInfo> = serde_json::from_value(data)?;
        Ok(likes)
    }

    pub async fn get_like_list(&self) -> Result<Vec<LikeInfo>> {
        self.get_like_list_with_timeout(Duration::from_secs(10))
            .await
    }
}

