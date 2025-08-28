
use crate::prelude::BotContext;
use crate::prelude::EchoAsyncResponse;
use serde_json::json;
use crate::error::Result;

impl BotContext {
    pub async fn delete_msg(&self, message_id: i64) -> Result<EchoAsyncResponse> {
        let msg = json!(
            {
                "message_id": message_id,
            }
        );
        self.websocket_send("delete_msg", msg).await
    }
}