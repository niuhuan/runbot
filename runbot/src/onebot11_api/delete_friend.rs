use crate::error::Result;
use crate::prelude::BotContext;
use crate::prelude::EchoAsyncResponse;

impl BotContext {
    pub async fn delete_friend(
        &self,
        user_id: i64,
        temp_block: bool,
        temp_both_del: bool,
    ) -> Result<EchoAsyncResponse> {
        self.websocket_send(
            "delete_friend",
            serde_json::json!({
                "user_id": user_id,
                "temp_block": temp_block,
                "temp_both_del": temp_both_del,
            }),
        )
        .await
    }
}
