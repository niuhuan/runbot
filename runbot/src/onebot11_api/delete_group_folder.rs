use crate::error::Result;
use crate::prelude::BotContext;
use crate::prelude::EchoAsyncResponse;
use serde_json::json;

impl BotContext {
    pub async fn delete_group_folder(
        &self,
        group_id: i64,
        folder_id: &str,
    ) -> Result<EchoAsyncResponse> {
        self.websocket_send(
            "delete_group_folder",
            json!({
                "group_id": group_id,
                "folder_id": folder_id,
            }),
        )
        .await
    }
}
