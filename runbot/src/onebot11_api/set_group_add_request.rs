use crate::error::Result;
use crate::prelude::BotContext;
use crate::prelude::EchoAsyncResponse;
use serde_json::json;

impl BotContext {
    pub async fn set_group_add_request(
        &self,
        flag: &str,
        approve: bool,
        reason: impl Into<Option<&str>>,
    ) -> Result<EchoAsyncResponse> {
        let msg = json!(
            {
                "flag": flag,
                "approve": approve,
                "remark": if let Some(r) = reason.into() { r } else { "" },
            }
        );
        self.websocket_send("set_group_add_request", msg).await
    }
}
