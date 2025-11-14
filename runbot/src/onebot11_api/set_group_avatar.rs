use crate::error::Result;
use crate::prelude::BotContext;
use serde_json::json;

impl BotContext {
    pub async fn set_group_avatar(&self, group_id: i64, file: &str, cache: i32) -> Result<()> {
        self.websocket_send(
            "set_group_avatar",
            json!({
                "group_id": group_id,
                "file": file,
                "cache": cache,
            }),
        )
        .await?;
        Ok(())
    }
}

