use crate::error::Result;
use crate::prelude::BotContext;
use serde_json::json;

impl BotContext {
    pub async fn set_group_card(&self, group_id: i64, user_id: i64, card: &str) -> Result<()> {
        let msg = json!(
            {
                "group_id": group_id,
                "user_id": user_id,
                "card": card,
            }
        );
        self.websocket_send("set_group_card", msg).await?;
        Ok(())
    }
}
