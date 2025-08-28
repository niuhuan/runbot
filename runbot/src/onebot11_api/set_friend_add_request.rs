use crate::error::Result;
use crate::prelude::BotContext;
use crate::prelude::EchoAsyncResponse;
use serde_json::json;

impl BotContext {
    pub async fn set_friend_add_request(
        &self,
        flag: &str,
        approve: bool,
        remark: Option<&str>,
    ) -> Result<EchoAsyncResponse> {
        let msg = json!(
            {
                "flag": flag,
                "approve": approve,
                "remark": remark,
            }
        );
        self.websocket_send("set_friend_add_request", msg).await
    }
}
