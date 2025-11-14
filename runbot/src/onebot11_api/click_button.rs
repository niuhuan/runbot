use crate::error::Result;
use crate::prelude::BotContext;
use serde_json::json;

impl BotContext {
    pub async fn click_button(&self, message_id: i64, button_id: &str) -> Result<()> {
        self.websocket_send(
            "click_button",
            json!({
                "message_id": message_id,
                "button_id": button_id,
            }),
        )
        .await?;
        Ok(())
    }
}

