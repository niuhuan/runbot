//! Fetches an image file.
//!
//! The request waits up to ten seconds for a response and returns the image file identifier
//! provided by the upstream.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use tokio::time::Duration;

/// Image file information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Image {
    /// Image file identifier, which may be a path, URL, or upstream-internal ID.
    pub file: String,
}

impl BotContext {
    /// Requests an image file and waits up to ten seconds for the response data.
    ///
    /// # Arguments
    ///
    /// - `file`: image file identifier, usually from the `file` field of a message segment.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when the current connection is unavailable
    /// or `retcode` is non-zero; returns [`crate::error::Error::WebSocketError`] when the
    /// WebSocket send fails; returns [`crate::error::Error::TimeoutError`] when no response
    /// arrives within ten seconds; returns [`crate::error::Error::JsonError`] when the response
    /// cannot be deserialized into [`Image`].
    pub async fn get_image(&self, file: &str) -> Result<Image> {
        let response = self
            .websocket_send(
                "get_image",
                serde_json::json!({
                    "file": file,
                }),
            )
            .await?;
        let response = response.data(Duration::from_secs(10)).await?;
        let image: Image = serde_json::from_value(response)?;
        Ok(image)
    }
}
