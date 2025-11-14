use crate::error::Result;
use crate::prelude::BotContext;
use crate::prelude::EchoAsyncResponse;
use serde_json::json;

impl BotContext {
    pub async fn send_private_forward_msg(
        &self,
        user_id: i64,
        messages: serde_json::Value,
    ) -> Result<EchoAsyncResponse> {
        self.websocket_send(
            "send_private_forward_msg",
            json!({
                "user_id": user_id,
                "messages": messages,
            }),
        )
        .await
    }

    pub async fn send_group_forward_msg(
        &self,
        group_id: i64,
        messages: serde_json::Value,
    ) -> Result<EchoAsyncResponse> {
        self.websocket_send(
            "send_group_forward_msg",
            json!({
                "group_id": group_id,
                "messages": messages,
            }),
        )
        .await
    }
}

