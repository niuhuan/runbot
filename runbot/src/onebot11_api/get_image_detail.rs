use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use tokio::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageDetail {
    pub size: i64,
    pub filename: String,
    pub url: String,
}

impl BotContext {
    pub async fn get_image_detail_with_timeout(
        &self,
        file: &str,
        timeout: Duration,
    ) -> Result<ImageDetail> {
        let response = self
            .websocket_send(
                "get_image_detail",
                json!({
                    "file": file,
                }),
            )
            .await?;
        let data = response.data(timeout).await?;
        let detail: ImageDetail = serde_json::from_value(data)?;
        Ok(detail)
    }

    pub async fn get_image_detail(&self, file: &str) -> Result<ImageDetail> {
        self.get_image_detail_with_timeout(file, Duration::from_secs(10))
            .await
    }
}

