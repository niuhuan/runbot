use crate::prelude::BotContext;
use serde_json::json;
use crate::error::Result;

impl BotContext {
    pub async fn set_group_admin(&self, group_id: i64, user_id: i64, enable: bool) -> Result<()> {
        let msg = json!(
            {
                "group_id": group_id,
                "user_id": user_id,
                "enable": enable,
            }
        );
        self.websocket_send("set_group_admin", msg).await?;
        Ok(())
    }
}