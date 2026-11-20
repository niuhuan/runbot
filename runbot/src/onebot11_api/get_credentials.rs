//! Get credentials for QQ-related interfaces.
//!
//! The default version waits up to ten seconds for the response; the `domain` field is not
//! sent when `None`.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use tokio::time::Duration;

/// Interface credentials.
///
/// The `Debug` output includes the raw credential values; callers should not write this
/// structure to logs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    /// Cookies string.
    pub cookies: String,
    /// CSRF token.
    pub csrf_token: i32,
}

impl BotContext {
    /// Request interface credentials and wait for the response data within the specified
    /// timeout.
    ///
    /// # Arguments
    ///
    /// - `domain`: Domain restriction; the `domain` field is not sent when `None`.
    /// - `timeout`: Response wait duration after the request has been sent.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when the current connection is
    /// unavailable or `retcode` is non-zero; returns
    /// [`crate::error::Error::WebSocketError`] when the WebSocket send fails; returns
    /// [`crate::error::Error::TimeoutError`] on timeout; returns
    /// [`crate::error::Error::JsonError`] when the response cannot be deserialized into
    /// [`Credentials`].
    pub async fn get_credentials_with_timeout(
        &self,
        domain: Option<&str>,
        timeout: Duration,
    ) -> Result<Credentials> {
        let mut params = serde_json::Map::new();
        if let Some(d) = domain {
            params.insert("domain".to_string(), json!(d));
        }
        let response = self
            .websocket_send("get_credentials", json!(params))
            .await?;
        let data = response.data(timeout).await?;
        let creds: Credentials = serde_json::from_value(data)?;
        Ok(creds)
    }

    /// Call [`Self::get_credentials_with_timeout`] with a ten-second response wait.
    ///
    /// See the timeout version for other behavior and errors.
    pub async fn get_credentials(&self, domain: Option<&str>) -> Result<Credentials> {
        self.get_credentials_with_timeout(domain, Duration::from_secs(10))
            .await
    }
}

