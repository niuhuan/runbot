use crate::error::Result;
use crate::prelude::BotContext;
use serde_json::json;

impl BotContext {
    pub async fn handle_filtered_friend_request(
        &self,
        request_id: i64,
        approve: bool,
        remark: Option<&str>,
    ) -> Result<()> {
        let mut params = json!({
            "request_id": request_id,
            "approve": approve,
        });
        if let Some(r) = remark {
            params["remark"] = json!(r);
        }
        self.websocket_send("handle_filtered_friend_request", params)
            .await?;
        Ok(())
    }
}

