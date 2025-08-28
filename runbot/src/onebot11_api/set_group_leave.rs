use crate::error::Result;
use crate::prelude::BotContext;
use serde_json::json;

impl BotContext {
    pub async fn set_group_leave(&self, group_id: i64, is_dismiss: bool) -> Result<()> {
        let msg = json!(
            {
                "group_id": group_id,
                "is_dismiss": is_dismiss,
            }
        );
        self.websocket_send("set_group_leave", msg).await?;
        Ok(())
    }
}
