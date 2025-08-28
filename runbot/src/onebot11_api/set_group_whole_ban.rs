use crate::error::Result;
use crate::prelude::BotContext;
use serde_json::json;

impl BotContext {
    pub async fn set_group_whole_ban(&self, group_id: i64, enable: bool) -> Result<()> {
        let msg = json!(
            {
                "group_id": group_id,
                "enable": enable,
            }
        );
        self.websocket_send("set_group_whole_ban", msg).await?;
        Ok(())
    }
}
