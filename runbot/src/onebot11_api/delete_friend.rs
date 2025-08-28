use crate::error::Result;
use crate::prelude::BotContext;

impl BotContext {
    pub async fn delete_friend(&self, user_id: i64) -> Result<()> {
        self.websocket_send(
            "delete_friend",
            serde_json::json!({
                "user_id": user_id,
            }),
        )
        .await?;
        Ok(())
    }
}
