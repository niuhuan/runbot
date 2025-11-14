use crate::error::Result;
use crate::prelude::BotContext;
use serde_json::json;

impl BotContext {
    pub async fn send_custom_packet(&self, command: &str, data: serde_json::Value) -> Result<()> {
        self.websocket_send(
            "send_custom_packet",
            json!({
                "command": command,
                "data": data,
            }),
        )
        .await?;
        Ok(())
    }
}

