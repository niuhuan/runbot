use crate::error::Result;
use crate::prelude::BotContext;
use serde_json::json;

impl BotContext {
    pub async fn set_group_msg_read(&self, group_id: i64, message_id: i64) -> Result<()> {
        self.websocket_send(
            "set_group_msg_read",
            json!({
                "group_id": group_id,
                "message_id": message_id,
            }),
        )
        .await?;
        Ok(())
    }
}

