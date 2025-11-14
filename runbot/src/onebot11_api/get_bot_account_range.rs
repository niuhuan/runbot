use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use tokio::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotAccountRange {
    pub min: i64,
    pub max: i64,
}

impl BotContext {
    pub async fn get_bot_account_range_with_timeout(
        &self,
        timeout: Duration,
    ) -> Result<BotAccountRange> {
        let response = self
            .websocket_send("get_bot_account_range", serde_json::Value::Null)
            .await?;
        let data = response.data(timeout).await?;
        let range: BotAccountRange = serde_json::from_value(data)?;
        Ok(range)
    }

    pub async fn get_bot_account_range(&self) -> Result<BotAccountRange> {
        self.get_bot_account_range_with_timeout(Duration::from_secs(10))
            .await
    }
}

