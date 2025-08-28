use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use tokio::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Friend {
    pub user_id: i64,
    pub nickname: String,
    #[serde(default)]
    pub remark: String,
}

impl BotContext {
    pub async fn get_friend_list_with_timeout(&self, timeout: Duration) -> Result<Vec<Friend>> {
        let response = self
            .websocket_send("get_friend_list", serde_json::Value::Null)
            .await?;
        let response = response.data(timeout).await?;
        let friend_list: Vec<Friend> = serde_json::from_value(response)?;
        Ok(friend_list)
    }

    pub async fn get_friend_list(&self) -> Result<Vec<Friend>> {
        self.get_friend_list_with_timeout(Duration::from_secs(10))
            .await
    }
}
