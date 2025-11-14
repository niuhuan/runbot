use crate::error::Result;
use crate::prelude::BotContext;
use crate::prelude::EchoAsyncResponse;
use serde_json::json;

impl BotContext {
    pub async fn send_group_ai_voice(
        &self,
        group_id: i64,
        text: &str,
        person: Option<i32>,
    ) -> Result<EchoAsyncResponse> {
        let mut params = json!({
            "group_id": group_id,
            "text": text,
        });
        if let Some(p) = person {
            params["person"] = json!(p);
        }
        self.websocket_send("send_group_ai_voice", params).await
    }
}

