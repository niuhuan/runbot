use crate::prelude::BotContext;
use serde_json::json;
use crate::error::Result;

impl BotContext {
    pub async fn set_group_special_title(&self, group_id: i64, user_id: i64, special_title: &str, duration: i64) -> Result<()> {
        let msg = json!(
            {
                "group_id": group_id,
                "user_id": user_id,
                "special_title": special_title,
                "duration": duration,
            }
        );
        self.websocket_send("set_group_special_title", msg).await?;
        Ok(())
    }
}