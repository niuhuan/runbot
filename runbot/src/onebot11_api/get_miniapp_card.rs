use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use tokio::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiniappCard {
    pub appid: String,
    pub appname: String,
    pub icon: String,
    pub view: String,
    pub desc: String,
    pub qrcode: String,
}

impl BotContext {
    pub async fn get_miniapp_card_with_timeout(
        &self,
        message_id: i64,
        timeout: Duration,
    ) -> Result<MiniappCard> {
        let response = self
            .websocket_send(
                "get_miniapp_card",
                json!({
                    "message_id": message_id,
                }),
            )
            .await?;
        let data = response.data(timeout).await?;
        let card: MiniappCard = serde_json::from_value(data)?;
        Ok(card)
    }

    pub async fn get_miniapp_card(&self, message_id: i64) -> Result<MiniappCard> {
        self.get_miniapp_card_with_timeout(message_id, Duration::from_secs(10))
            .await
    }
}

