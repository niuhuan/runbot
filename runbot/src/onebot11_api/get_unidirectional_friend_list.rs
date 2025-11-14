use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use tokio::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnidirectionalFriend {
    pub user_id: i64,
    pub nickname: String,
    pub source: String,
}

impl BotContext {
    pub async fn get_unidirectional_friend_list_with_timeout(
        &self,
        timeout: Duration,
    ) -> Result<Vec<UnidirectionalFriend>> {
        let response = self
            .websocket_send("get_unidirectional_friend_list", serde_json::Value::Null)
            .await?;
        let data = response.data(timeout).await?;
        let friends: Vec<UnidirectionalFriend> = serde_json::from_value(data)?;
        Ok(friends)
    }

    pub async fn get_unidirectional_friend_list(&self) -> Result<Vec<UnidirectionalFriend>> {
        self.get_unidirectional_friend_list_with_timeout(Duration::from_secs(10))
            .await
    }
}

