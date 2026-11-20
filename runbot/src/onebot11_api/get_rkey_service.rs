//! Get rkey service information.
//!
//! The default version waits up to ten seconds for the response; `url` is sent to the
//! upstream as-is.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use tokio::time::Duration;

/// rkey service response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RKeyService {
    /// rkey string returned by the upstream.
    pub rkey: String,
}

impl BotContext {
    /// Request rkey service information and wait for the response data within the specified
    /// timeout.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when the current connection is
    /// unavailable or `retcode` is non-zero; returns
    /// [`crate::error::Error::WebSocketError`] when the WebSocket send fails; returns
    /// [`crate::error::Error::TimeoutError`] on timeout; returns
    /// [`crate::error::Error::JsonError`] when the response cannot be deserialized into
    /// [`RKeyService`].
    pub async fn get_rkey_service_with_timeout(
        &self,
        url: &str,
        timeout: Duration,
    ) -> Result<RKeyService> {
        let response = self
            .websocket_send(
                "get_rkey_service",
                json!({
                    "url": url,
                }),
            )
            .await?;
        let data = response.data(timeout).await?;
        let rkey: RKeyService = serde_json::from_value(data)?;
        Ok(rkey)
    }

    /// Call [`Self::get_rkey_service_with_timeout`] with a ten-second response wait.
    ///
    /// See the timeout version for other behavior and errors.
    pub async fn get_rkey_service(&self, url: &str) -> Result<RKeyService> {
        self.get_rkey_service_with_timeout(url, Duration::from_secs(10))
            .await
    }
}

