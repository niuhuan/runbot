use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use tokio::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordDetail {
    pub file: String,
    pub magic: i32,
}

impl BotContext {
    pub async fn get_record_detail_with_timeout(
        &self,
        file: &str,
        timeout: Duration,
    ) -> Result<RecordDetail> {
        let response = self
            .websocket_send(
                "get_record_detail",
                json!({
                    "file": file,
                }),
            )
            .await?;
        let data = response.data(timeout).await?;
        let detail: RecordDetail = serde_json::from_value(data)?;
        Ok(detail)
    }

    pub async fn get_record_detail(&self, file: &str) -> Result<RecordDetail> {
        self.get_record_detail_with_timeout(file, Duration::from_secs(10))
            .await
    }
}

