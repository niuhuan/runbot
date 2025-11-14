use crate::error::Result;
use crate::prelude::BotContext;
use serde_json::json;

impl BotContext {
    pub async fn set_signature(&self, signature: &str) -> Result<()> {
        self.websocket_send(
            "set_signature",
            json!({
                "signature": signature,
            }),
        )
        .await?;
        Ok(())
    }
}

