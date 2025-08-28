use std::time::Duration;

use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrImageCoordinate {
    pub x: i64,
    pub y: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrImageText {
    pub text: String,
    #[serde(default)]
    pub confidence: i64,
    pub coordinates: Vec<OcrImageCoordinate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrImageResponse {
    pub text: Vec<OcrImageText>,
    #[serde(default)]
    pub language: String,
}

impl BotContext {
    pub async fn ocr_image_with_timeout(
        &self,
        image: &str,
        timeout: Duration,
    ) -> Result<OcrImageResponse> {
        let response = self
            .websocket_send(
                "ocr_image",
                serde_json::json!({
                    "image": image,
                }),
            )
            .await?;
        let response: OcrImageResponse = serde_json::from_value(response.data(timeout).await?)?;
        Ok(response)
    }

    pub async fn ocr_image(&self, image: &str) -> Result<OcrImageResponse> {
        self.ocr_image_with_timeout(image, Duration::from_secs(10))
            .await
    }
}
