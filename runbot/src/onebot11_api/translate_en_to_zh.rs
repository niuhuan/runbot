use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use tokio::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslateResult {
    pub text: String,
}

impl BotContext {
    pub async fn translate_en_to_zh_with_timeout(
        &self,
        text: &str,
        timeout: Duration,
    ) -> Result<TranslateResult> {
        let response = self
            .websocket_send(
                "translate_en_to_zh",
                json!({
                    "text": text,
                }),
            )
            .await?;
        let data = response.data(timeout).await?;
        let result: TranslateResult = serde_json::from_value(data)?;
        Ok(result)
    }

    pub async fn translate_en_to_zh(&self, text: &str) -> Result<TranslateResult> {
        self.translate_en_to_zh_with_timeout(text, Duration::from_secs(10))
            .await
    }
}

