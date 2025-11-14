use crate::error::Result;
use crate::prelude::BotContext;
use serde_json::json;

impl BotContext {
    pub async fn move_group_file(
        &self,
        group_id: i64,
        file_id: &str,
        folder_id: &str,
    ) -> Result<()> {
        self.websocket_send(
            "move_group_file",
            json!({
                "group_id": group_id,
                "file_id": file_id,
                "folder_id": folder_id,
            }),
        )
        .await?;
        Ok(())
    }
}

