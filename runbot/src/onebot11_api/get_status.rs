use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use tokio::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Status {
    pub online: bool,
    pub good: bool,
}

impl BotContext {
    pub async fn get_status_with_timeout(&self, timeout: Duration) -> Result<Status> {
        let response = self
            .websocket_send("get_status", serde_json::Value::Null)
            .await?;
        let data = response.data(timeout).await?;
        let status: Status = serde_json::from_value(data)?;
        Ok(status)
    }

    pub async fn get_status(&self) -> Result<Status> {
        self.get_status_with_timeout(Duration::from_secs(10))
            .await
    }
}

