use crate::error::Result;
use crate::prelude::BotContext;
use serde_json::json;

impl BotContext {
    pub async fn rename_group_file(
        &self,
        group_id: i64,
        file_id: &str,
        name: &str,
    ) -> Result<()> {
        self.websocket_send(
            "rename_group_file",
            json!({
                "group_id": group_id,
                "file_id": file_id,
                "name": name,
            }),
        )
        .await?;
        Ok(())
    }
}

