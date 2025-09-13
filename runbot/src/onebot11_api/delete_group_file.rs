use crate::error::Result;
use crate::prelude::BotContext;
use crate::prelude::EchoAsyncResponse;
use serde_json::json;

impl BotContext {
    pub async fn delete_group_file(
        &self,
        group_id: i64,
        file_id: &str,
    ) -> Result<EchoAsyncResponse> {
        self.websocket_send(
            "delete_group_file",
            json!({
                "group_id": group_id,
                "file_id": file_id,
            }),
        )
        .await
    }
}
