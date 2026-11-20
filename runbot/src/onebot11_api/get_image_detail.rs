//! Fetches image message details.
//!
//! The request returns the image size, filename, and URL within the specified response wait
//! time.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use tokio::time::Duration;

/// Image message details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageDetail {
    /// Image size; the unit is defined by the upstream (usually bytes).
    pub size: i64,
    /// File name.
    pub filename: String,
    /// Image URL.
    pub url: String,
}

impl BotContext {
    /// Requests image message details and waits up to the given time for the response data.
    ///
    /// `timeout` only limits the response wait after the request has been sent; it does not
    /// include the send phase.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when the current connection is unavailable
    /// or `retcode` is non-zero; returns [`crate::error::Error::WebSocketError`] when the
    /// WebSocket send fails; returns [`crate::error::Error::TimeoutError`] on timeout; returns
    /// [`crate::error::Error::JsonError`] when the response cannot be deserialized into
    /// [`ImageDetail`].
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

    /// Calls [`Self::get_image_detail_with_timeout`] with a ten-second response wait.
    ///
    /// See the timeout variant for other behavior and errors.
    pub async fn get_image_detail(&self, file: &str) -> Result<ImageDetail> {
        self.get_image_detail_with_timeout(file, Duration::from_secs(10))
            .await
    }
}

