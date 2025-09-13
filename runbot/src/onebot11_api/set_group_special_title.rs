use crate::error::Result;
use crate::prelude::BotContext;
use serde_json::json;

impl BotContext {
    pub async fn set_group_special_title(
        &self,
        group_id: i64,
        user_id: i64,
        special_title: &str,
    ) -> Result<()> {
        let msg = json!(
            {
                "group_id": group_id,
                "user_id": user_id,
                "special_title": special_title
            }
        );
        self.websocket_send("set_group_special_title", msg).await?;
        Ok(())
    }
}
