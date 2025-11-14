use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use tokio::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub size: i64,
    pub filename: String,
    pub file_id: String,
}

impl BotContext {
    pub async fn get_file_info_with_timeout(
        &self,
        file_id: &str,
        timeout: Duration,
    ) -> Result<FileInfo> {
        let response = self
            .websocket_send(
                "get_file_info",
                json!({
                    "file_id": file_id,
                }),
            )
            .await?;
        let data = response.data(timeout).await?;
        let info: FileInfo = serde_json::from_value(data)?;
        Ok(info)
    }

    pub async fn get_file_info(&self, file_id: &str) -> Result<FileInfo> {
        self.get_file_info_with_timeout(file_id, Duration::from_secs(10))
            .await
    }
}

