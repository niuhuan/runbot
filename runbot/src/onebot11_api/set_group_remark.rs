use crate::error::Result;
use crate::prelude::BotContext;

impl BotContext {
    pub async fn set_group_remark(&self, group_id: i64, remark: &str) -> Result<()> {
        self.websocket_send(
            "set_group_remark",
            serde_json::json!({
                "group_id": group_id,
                "remark": remark,
            }),
        )
        .await?;
        Ok(())
    }
}
