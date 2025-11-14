use crate::error::Result;
use crate::prelude::BotContext;
use serde_json::json;

impl BotContext {
    pub async fn create_favorite(&self, message_id: i64) -> Result<()> {
        self.websocket_send(
            "create_favorite",
            json!({
                "message_id": message_id,
            }),
        )
        .await?;
        Ok(())
    }
}

