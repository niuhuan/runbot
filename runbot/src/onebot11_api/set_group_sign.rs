use crate::error::Result;
use crate::prelude::BotContext;
use crate::prelude::EchoAsyncResponse;
use serde_json::json;

impl BotContext {
    pub async fn set_group_sign(&self, group_id: i64) -> Result<EchoAsyncResponse> {
        let msg = json!(
            {
                "group_id": group_id,
            }
        );
        self.websocket_send("set_group_sign", msg).await
    }
    pub async fn send_group_sign(&self, group_id: i64) -> Result<EchoAsyncResponse> {
        let msg = json!(
            {
                "group_id": group_id,
            }
        );
        self.websocket_send("send_group_sign", msg).await
    }
}
