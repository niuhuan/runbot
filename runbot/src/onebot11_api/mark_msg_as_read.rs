use crate::prelude::BotContext;
use crate::error::Result;

impl BotContext {
    pub async fn mark_msg_as_read(&self, message_id: i64) -> Result<()> {
        self.websocket_send("mark_msg_as_read", serde_json::json!({
            "message_id": message_id,
        })).await?;
        Ok(())
    }
}