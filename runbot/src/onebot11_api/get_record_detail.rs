//! Get voice message details.
//!
//! The default version waits up to ten seconds for the response and returns the file
//! identifier and the `magic` value supplied by the upstream.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use tokio::time::Duration;

/// Voice message details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordDetail {
    /// Voice file identifier.
    pub file: String,
    /// Format or magic identifier supplied by the upstream; the exact meaning is
    /// implementation-defined.
    pub magic: i32,
}

impl BotContext {
    /// Request voice message details and wait for the response data within the specified
    /// timeout.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when the current connection is
    /// unavailable or `retcode` is non-zero; returns
    /// [`crate::error::Error::WebSocketError`] when the WebSocket send fails; returns
    /// [`crate::error::Error::TimeoutError`] on timeout; returns
    /// [`crate::error::Error::JsonError`] when the response cannot be deserialized into
    /// [`RecordDetail`].
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

    /// Call [`Self::get_record_detail_with_timeout`] with a ten-second response wait.
    ///
    /// See the timeout version for other behavior and errors.
    pub async fn get_record_detail(&self, file: &str) -> Result<RecordDetail> {
        self.get_record_detail_with_timeout(file, Duration::from_secs(10))
            .await
    }
}

