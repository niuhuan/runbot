use serde_json::json;
use crate::error::Result;
use crate::prelude::BotContext;

impl BotContext {
    pub async fn set_group_ban(&self, group_id: i64, user_id: i64, duration: i64) -> Result<()> {
        let msg = json!(
            {
                "group_id": group_id,
                "user_id": user_id,
                "duration": duration,
            }
        );
        self.websocket_send("set_group_ban", msg).await?;
        Ok(())
    }
}
