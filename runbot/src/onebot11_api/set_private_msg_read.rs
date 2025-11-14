use crate::error::Result;
use crate::prelude::BotContext;
use serde_json::json;

impl BotContext {
    pub async fn set_private_msg_read(&self, user_id: i64, message_id: i64) -> Result<()> {
        self.websocket_send(
            "set_private_msg_read",
            json!({
                "user_id": user_id,
                "message_id": message_id,
            }),
        )
        .await?;
        Ok(())
    }
}

