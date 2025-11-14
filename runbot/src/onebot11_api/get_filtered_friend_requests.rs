use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use tokio::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilteredFriendRequest {
    pub request_id: i64,
    pub requester_nick: String,
    pub requester_id: i64,
    pub request_message: String,
    pub request_time: i64,
}

impl BotContext {
    pub async fn get_filtered_friend_requests_with_timeout(
        &self,
        timeout: Duration,
    ) -> Result<Vec<FilteredFriendRequest>> {
        let response = self
            .websocket_send("get_filtered_friend_requests", serde_json::Value::Null)
            .await?;
        let data = response.data(timeout).await?;
        let requests: Vec<FilteredFriendRequest> = serde_json::from_value(data)?;
        Ok(requests)
    }

    pub async fn get_filtered_friend_requests(&self) -> Result<Vec<FilteredFriendRequest>> {
        self.get_filtered_friend_requests_with_timeout(Duration::from_secs(10))
            .await
    }
}

