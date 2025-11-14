use crate::error::Result;
use crate::prelude::BotContext;
use serde_json::json;

impl BotContext {
    pub async fn set_online_status(&self, status: i32) -> Result<()> {
        self.websocket_send(
            "set_online_status",
            json!({
                "status": status,
            }),
        )
        .await?;
        Ok(())
    }
}

