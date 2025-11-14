use crate::error::Result;
use crate::prelude::BotContext;
use serde_json::json;

impl BotContext {
    pub async fn set_input_status(&self, user_id: i64, typing: bool) -> Result<()> {
        self.websocket_send(
            "set_input_status",
            json!({
                "user_id": user_id,
                "typing": typing,
            }),
        )
        .await?;
        Ok(())
    }
}

