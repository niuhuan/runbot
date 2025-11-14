use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use tokio::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadedFile {
    pub file: String,
}

impl BotContext {
    pub async fn download_file_to_cache_with_timeout(
        &self,
        url: &str,
        thread_count: i32,
        headers: Option<serde_json::Value>,
        timeout: Duration,
    ) -> Result<DownloadedFile> {
        let mut params = json!({
            "url": url,
            "thread_count": thread_count,
        });
        if let Some(h) = headers {
            params["headers"] = h;
        }
        let response = self
            .websocket_send("download_file_to_cache", params)
            .await?;
        let data = response.data(timeout).await?;
        let file: DownloadedFile = serde_json::from_value(data)?;
        Ok(file)
    }

    pub async fn download_file_to_cache(
        &self,
        url: &str,
        thread_count: i32,
        headers: Option<serde_json::Value>,
    ) -> Result<DownloadedFile> {
        self.download_file_to_cache_with_timeout(url, thread_count, headers, Duration::from_secs(30))
            .await
    }
}

