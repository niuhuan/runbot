use crate::error::Result;
use crate::prelude::BotContext;
use serde_json::json;

impl BotContext {
    pub async fn send_like(&self, user_id: i64, times: i32) -> Result<()> {
        self.websocket_send(
            "send_like",
            json!({
                "user_id": user_id,
                "times": times,
            }),
        )
        .await?;
        Ok(())
    }
}

