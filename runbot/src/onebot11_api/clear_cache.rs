use crate::error::Result;
use crate::prelude::BotContext;

impl BotContext {
    pub async fn clear_cache(&self) -> Result<()> {
        self.websocket_send("clear_cache", serde_json::Value::Null)
            .await?;
        Ok(())
    }
}

