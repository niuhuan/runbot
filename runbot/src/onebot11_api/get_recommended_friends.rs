use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use tokio::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendedFriend {
    pub user_id: i64,
    pub nickname: String,
    pub sex: String,
    pub age: i64,
    pub qid: String,
    pub brief: String,
    pub school: String,
}

impl BotContext {
    pub async fn get_recommended_friends_with_timeout(
        &self,
        timeout: Duration,
    ) -> Result<Vec<RecommendedFriend>> {
        let response = self
            .websocket_send("get_recommended_friends", serde_json::Value::Null)
            .await?;
        let data = response.data(timeout).await?;
        let friends: Vec<RecommendedFriend> = serde_json::from_value(data)?;
        Ok(friends)
    }

    pub async fn get_recommended_friends(&self) -> Result<Vec<RecommendedFriend>> {
        self.get_recommended_friends_with_timeout(Duration::from_secs(10))
            .await
    }
}

