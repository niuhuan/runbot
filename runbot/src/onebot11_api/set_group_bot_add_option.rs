use crate::error::Result;
use crate::prelude::BotContext;
use serde_json::json;

impl BotContext {
    pub async fn set_group_bot_add_option(&self, group_id: i64, approve: bool) -> Result<()> {
        self.websocket_send(
            "set_group_bot_add_option",
            json!({
                "group_id": group_id,
                "approve": approve,
            }),
        )
        .await?;
        Ok(())
    }
}

