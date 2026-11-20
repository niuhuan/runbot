//! Image OCR.
//!
//! The default version waits up to ten seconds for a response and returns the recognized
//! text, confidence, and coordinates.

use std::time::Duration;

use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};

/// Coordinate point of an OCR text region; the unit is defined by the upstream (usually
/// pixels).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrImageCoordinate {
    /// X coordinate.
    pub x: i64,
    /// Y coordinate.
    pub y: i64,
}

/// A single OCR recognition result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrImageText {
    /// Recognized text.
    pub text: String,
    /// Confidence; `0` when missing; the value range is defined by the upstream.
    #[serde(default)]
    pub confidence: i64,
    /// Text region coordinates, stored in the order returned by the upstream.
    pub coordinates: Vec<OcrImageCoordinate>,
}

/// Image OCR result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrImageResponse {
    /// Recognized text segments.
    pub text: Vec<OcrImageText>,
    /// Recognition language; empty string when missing.
    #[serde(default)]
    pub language: String,
}

impl BotContext {
    /// Requests OCR on an image and waits up to the given time for the response data.
    ///
    /// # Arguments
    ///
    /// - `image`: image file identifier or another reference form accepted by the upstream;
    ///   sent as is by this method.
    /// - `timeout`: response wait duration after the request has been sent.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when the current connection is unavailable
    /// or `retcode` is non-zero; returns [`crate::error::Error::WebSocketError`] when the
    /// WebSocket send fails; returns [`crate::error::Error::TimeoutError`] on timeout; returns
    /// [`crate::error::Error::JsonError`] when the response cannot be deserialized into
    /// [`OcrImageResponse`].
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

    /// Calls [`Self::ocr_image_with_timeout`] with a ten-second response wait.
    ///
    /// See the timeout variant for other behavior and errors.
    pub async fn ocr_image(&self, image: &str) -> Result<OcrImageResponse> {
        self.ocr_image_with_timeout(image, Duration::from_secs(10))
            .await
    }
}
