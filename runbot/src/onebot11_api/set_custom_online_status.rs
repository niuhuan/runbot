use crate::error::Result;
use crate::prelude::BotContext;
use serde_json::json;

impl BotContext {
    pub async fn set_custom_online_status(
        &self,
        status_text: &str,
        status_type: i32,
    ) -> Result<()> {
        self.websocket_send(
            "set_custom_online_status",
            json!({
                "status_text": status_text,
                "status_type": status_type,
            }),
        )
        .await?;
        Ok(())
    }
}

