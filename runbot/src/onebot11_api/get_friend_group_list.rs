use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use tokio::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriendGroup {
    pub group_id: i32,
    pub group_name: String,
    pub friend_count: i32,
}

impl BotContext {
    pub async fn get_friend_group_list_with_timeout(
        &self,
        timeout: Duration,
    ) -> Result<Vec<FriendGroup>> {
        let response = self
            .websocket_send("get_friend_group_list", serde_json::Value::Null)
            .await?;
        let data = response.data(timeout).await?;
        let groups: Vec<FriendGroup> = serde_json::from_value(data)?;
        Ok(groups)
    }

    pub async fn get_friend_group_list(&self) -> Result<Vec<FriendGroup>> {
        self.get_friend_group_list_with_timeout(Duration::from_secs(10))
            .await
    }
}

