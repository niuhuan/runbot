use crate::error::Result;
use crate::prelude::BotContext;
use serde_json::json;

impl BotContext {
    pub async fn set_group_search(&self, group_id: i64, enable: bool) -> Result<()> {
        self.websocket_send(
            "set_group_search",
            json!({
                "group_id": group_id,
                "enable": enable,
            }),
        )
        .await?;
        Ok(())
    }
}

