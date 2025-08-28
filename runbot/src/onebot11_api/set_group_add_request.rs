use crate::error::Result;
use crate::prelude::BotContext;
use crate::prelude::EchoAsyncResponse;
use crate::prelude::GroupRequestSubType;
use serde_json::json;

impl BotContext {
    pub async fn set_group_add_request(
        &self,
        flag: &str,
        approve: bool,
        sub_type: GroupRequestSubType,
        remark: Option<&str>,
    ) -> Result<EchoAsyncResponse> {
        let msg = json!(
            {
                "flag": flag,
                "approve": approve,
                "sub_type": sub_type,
                "remark": remark,
            }
        );
        self.websocket_send("set_group_add_request", msg).await
    }
}
