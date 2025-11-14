use crate::error::Result;
use crate::prelude::BotContext;
use serde_json::json;

impl BotContext {
    pub async fn set_essence_msg(&self, message_id: i64) -> Result<()> {
        self.websocket_send(
            "set_essence_msg",
            json!({
                "message_id": message_id,
            }),
        )
        .await?;
        Ok(())
    }
}

