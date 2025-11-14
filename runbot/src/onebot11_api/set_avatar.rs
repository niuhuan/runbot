use crate::error::Result;
use crate::prelude::BotContext;
use serde_json::json;

impl BotContext {
    pub async fn set_avatar(&self, file: &str) -> Result<()> {
        self.websocket_send(
            "set_avatar",
            json!({
                "file": file,
            }),
        )
        .await?;
        Ok(())
    }
}

