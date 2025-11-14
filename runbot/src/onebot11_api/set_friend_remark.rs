use crate::error::Result;
use crate::prelude::BotContext;
use serde_json::json;

impl BotContext {
    pub async fn set_friend_remark(&self, user_id: i64, remark: &str) -> Result<()> {
        self.websocket_send(
            "set_friend_remark",
            json!({
                "user_id": user_id,
                "remark": remark,
            }),
        )
        .await?;
        Ok(())
    }
}

