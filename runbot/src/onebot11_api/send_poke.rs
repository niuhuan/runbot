use crate::error::Result;
use crate::prelude::BotContext;

impl BotContext {
    pub async fn friend_poke(&self, user_id: i64) -> Result<()> {
        self.websocket_send(
            "friend_poke",
            serde_json::json!({
                "user_id": user_id,
            }),
        )
        .await?;
        Ok(())
    }
    pub async fn group_poke(&self, group_id: i64, user_id: i64) -> Result<()> {
        self.websocket_send(
            "group_poke",
            serde_json::json!({
                "group_id": group_id,
                "user_id": user_id,
            }),
        )
        .await?;
        Ok(())
    }
    pub async fn send_poke(&self, group_id: i64, user_id: i64) -> Result<()> {
        self.websocket_send(
            "send_poke",
            serde_json::json!({
                "group_id": group_id,
                "user_id": user_id,
            }),
        )
        .await?;
        Ok(())
    }
}
