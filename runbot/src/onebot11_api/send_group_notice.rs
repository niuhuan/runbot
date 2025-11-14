use crate::error::Result;
use crate::prelude::BotContext;
use serde_json::json;

impl BotContext {
    pub async fn send_group_notice(&self, group_id: i64, content: &str) -> Result<()> {
        self.websocket_send(
            "send_group_notice",
            json!({
                "group_id": group_id,
                "content": content,
            }),
        )
        .await?;
        Ok(())
    }
}

