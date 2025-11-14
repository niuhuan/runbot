use crate::error::Result;
use crate::prelude::BotContext;
use serde_json::json;

impl BotContext {
    pub async fn set_group_add_option(&self, group_id: i64, sub_type: &str, approve: bool) -> Result<()> {
        self.websocket_send(
            "set_group_add_option",
            json!({
                "group_id": group_id,
                "sub_type": sub_type,
                "approve": approve,
            }),
        )
        .await?;
        Ok(())
    }
}

