use crate::error::Result;
use crate::prelude::BotContext;
use crate::prelude::EchoAsyncResponse;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadPrivateFile {
    pub file_id: String,
}

pub struct UploadPrivateFileWaiting(EchoAsyncResponse);

impl UploadPrivateFileWaiting {
    pub async fn wait(self, timeout: Duration) -> Result<UploadPrivateFile> {
        let response = self.0.data(timeout).await?;
        let data: UploadPrivateFile = serde_json::from_value(response)?;
        Ok(data)
    }
}

impl BotContext {
    pub async fn upload_private_file(
        &self,
        user_id: i64,
        file: &str,
        name: &str,
    ) -> Result<UploadPrivateFileWaiting> {
        let response = self
            .websocket_send(
                "upload_private_file",
                json!({
                    "user_id": user_id,
                    "file": file,
                    "name": name,
                }),
            )
            .await?;
        Ok(UploadPrivateFileWaiting(response))
    }
}

