use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use tokio::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedFile {
    pub file: String,
}

impl BotContext {
    pub async fn save_file_to_cache_with_timeout(
        &self,
        file_id: &str,
        timeout: Duration,
    ) -> Result<SavedFile> {
        let response = self
            .websocket_send(
                "save_file_to_cache",
                json!({
                    "file_id": file_id,
                }),
            )
            .await?;
        let data = response.data(timeout).await?;
        let saved: SavedFile = serde_json::from_value(data)?;
        Ok(saved)
    }

    pub async fn save_file_to_cache(&self, file_id: &str) -> Result<SavedFile> {
        self.save_file_to_cache_with_timeout(file_id, Duration::from_secs(10))
            .await
    }
}

