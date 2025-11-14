use crate::error::Result;
use crate::prelude::BotContext;

impl BotContext {
    pub async fn account_logout(&self) -> Result<()> {
        self.websocket_send("account_logout", serde_json::Value::Null)
            .await?;
        Ok(())
    }
}

