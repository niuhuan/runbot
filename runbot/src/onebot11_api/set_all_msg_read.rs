use crate::error::Result;
use crate::prelude::BotContext;

impl BotContext {
    pub async fn set_all_msg_read(&self) -> Result<()> {
        self.websocket_send("set_all_msg_read", serde_json::Value::Null)
            .await?;
        Ok(())
    }
}

