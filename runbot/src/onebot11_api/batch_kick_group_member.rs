use crate::error::Result;
use crate::prelude::BotContext;
use serde_json::json;

impl BotContext {
    pub async fn batch_kick_group_member(
        &self,
        group_id: i64,
        user_ids: Vec<i64>,
        reject_add_request: bool,
    ) -> Result<()> {
        self.websocket_send(
            "batch_kick_group_member",
            json!({
                "group_id": group_id,
                "user_ids": user_ids,
                "reject_add_request": reject_add_request,
            }),
        )
        .await?;
        Ok(())
    }
}

