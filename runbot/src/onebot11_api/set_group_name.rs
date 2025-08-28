use crate::prelude::BotContext;
use serde_json::json;
use crate::error::Result;

impl BotContext {
    pub async fn set_group_name(&self, group_id: i64, group_name: &str) -> Result<()> {
        let msg = json!(
            {
                "group_id": group_id,
                "group_name": group_name,
            }
        );
        self.websocket_send("set_group_name", msg).await?;
        Ok(())
    }
}