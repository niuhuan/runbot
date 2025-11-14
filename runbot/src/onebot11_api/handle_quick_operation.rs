use crate::error::Result;
use crate::prelude::BotContext;
use serde_json::json;

impl BotContext {
    pub async fn handle_quick_operation(
        &self,
        context: serde_json::Value,
        operation: serde_json::Value,
    ) -> Result<()> {
        self.websocket_send(
            "handle_quick_operation",
            json!({
                "context": context,
                "operation": operation,
            }),
        )
        .await?;
        Ok(())
    }
}

