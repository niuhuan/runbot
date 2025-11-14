use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use tokio::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentContact {
    pub user_id: Option<i64>,
    pub group_id: Option<i64>,
    pub time: i64,
}

impl BotContext {
    pub async fn get_recent_contact_with_timeout(
        &self,
        timeout: Duration,
    ) -> Result<Vec<RecentContact>> {
        let response = self
            .websocket_send("get_recent_contact", serde_json::Value::Null)
            .await?;
        let data = response.data(timeout).await?;
        let contacts: Vec<RecentContact> = serde_json::from_value(data)?;
        Ok(contacts)
    }

    pub async fn get_recent_contact(&self) -> Result<Vec<RecentContact>> {
        self.get_recent_contact_with_timeout(Duration::from_secs(10))
            .await
    }
}

