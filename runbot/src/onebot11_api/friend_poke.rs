use crate::prelude::BotContext;
use crate::error::Result;

impl BotContext {
    pub async fn friend_poke(&self, user_id: i64) -> Result<()> {
        self.websocket_send("friend_poke", serde_json::json!({
            "user_id": user_id,
        })).await?;
        Ok(())
    }
}