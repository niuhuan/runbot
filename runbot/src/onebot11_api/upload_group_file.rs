use std::time::Duration;

use crate::error::Result;
use crate::prelude::BotContext;
use crate::prelude::EchoAsyncResponse;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadGroupFile {
    pub file_id: String,
}

pub struct UploadGroupFileWating(EchoAsyncResponse);

impl UploadGroupFileWating {
    pub async fn wait(self, timeout: Duration) -> Result<UploadGroupFile> {
        let response = self.0.data(timeout).await?;
        let can_send_record: UploadGroupFile = serde_json::from_value(response)?;
        Ok(can_send_record)
    }
}

impl BotContext {
    pub async fn upload_group_file_with(
        &self,
        group_id: i64,
        file: &str,
        name: &str,
        folder_id: impl Into<Option<&str>>,
    ) -> Result<UploadGroupFileWating> {
        let response = self
            .websocket_send(
                "upload_group_file",
                json!({
                    "group_id": group_id,
                    "file": file,
                    "name": name,
                    "folder_id": folder_id.into(),
                }),
            )
            .await?;
        Ok(UploadGroupFileWating(response))
    }
}
