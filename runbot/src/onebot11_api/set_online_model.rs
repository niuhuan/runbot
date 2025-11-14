use crate::error::Result;
use crate::prelude::BotContext;
use serde_json::json;

impl BotContext {
    pub async fn set_online_model(&self, model: &str) -> Result<()> {
        self.websocket_send(
            "set_online_model",
            json!({
                "model": model,
            }),
        )
        .await?;
        Ok(())
    }
}

