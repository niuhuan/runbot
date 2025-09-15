use crate::error::Result;
use crate::prelude::BotContext;
use crate::prelude::EchoAsyncResponse;
use serde_json::json;

impl BotContext {
    pub async fn rename_group_file_folder(
        &self,
        group_id: i64,
        folder_id: &str,
        new_folder_name: &str,
    ) -> Result<EchoAsyncResponse> {
        self.websocket_send(
            "rename_group_file_folder",
            json!({
                "group_id": group_id,
                "folder_id": folder_id,
                "new_folder_name": new_folder_name,
            }),
        )
        .await
    }
}
