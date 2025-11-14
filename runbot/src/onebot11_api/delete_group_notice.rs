use crate::error::Result;
use crate::prelude::BotContext;
use serde_json::json;

impl BotContext {
    pub async fn delete_group_notice(&self, group_id: i64, notice_id: &str) -> Result<()> {
        self.websocket_send(
            "delete_group_notice",
            json!({
                "group_id": group_id,
                "notice_id": notice_id,
            }),
        )
        .await?;
        Ok(())
    }
}

