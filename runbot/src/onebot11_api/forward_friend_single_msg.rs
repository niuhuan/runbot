use crate::error::Result;
use crate::prelude::BotContext;
use crate::prelude::EchoAsyncResponse;
use serde_json::json;

impl BotContext {
    pub async fn forward_friend_single_msg(
        &self,
        message_id: i64,
        user_id: i64,
    ) -> Result<EchoAsyncResponse> {
        let msg = json!(
            {
                "message_id": message_id,
                "user_id": user_id,
            }
        );
        self.websocket_send("forward_friend_single_msg", msg).await
    }
}
