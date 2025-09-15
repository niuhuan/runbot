use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use tokio::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanSendRecord {
    pub yes: bool,
}

impl BotContext {
    pub async fn can_send_record(&self) -> Result<CanSendRecord> {
        let response = self
            .websocket_send("can_send_record", serde_json::Value::Null)
            .await?;
        let response = response.data(Duration::from_secs(10)).await?;
        let data: CanSendRecord = serde_json::from_value(response)?;
        Ok(data)
    }
}
