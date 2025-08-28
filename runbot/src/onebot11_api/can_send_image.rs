use serde_derive::{Serialize, Deserialize};
use crate::prelude::BotContext;
use crate::error::Result;
use tokio::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanSendImage {
    pub yes: bool,
}

impl BotContext {
    pub async fn can_send_image(&self) -> Result<CanSendImage> {
        let response = self.websocket_send("can_send_image", serde_json::Value::Null).await?;
        let response = response.data(Duration::from_secs(10)).await?;
        let can_send_image: CanSendImage = serde_json::from_value(response)?;
        Ok(can_send_image)
    }
}