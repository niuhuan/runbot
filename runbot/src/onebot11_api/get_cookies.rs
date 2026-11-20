//! Get the cookies string.
//!
//! The default version waits up to ten seconds for the response; the `domain` field is not
//! sent when `None`.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use tokio::time::Duration;

/// Cookies response.
///
/// The `Debug` output includes the raw value of this field; callers should not write real
/// credentials to logs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cookies {
    /// Cookies string returned by the upstream.
    pub cookies: String,
}

impl BotContext {
    /// Request cookies and wait for the response data within the specified timeout.
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
    /// [`Cookies`].
    pub async fn get_cookies_with_timeout(
        &self,
        domain: Option<&str>,
        timeout: Duration,
    ) -> Result<Cookies> {
        let mut params = serde_json::Map::new();
        if let Some(d) = domain {
            params.insert("domain".to_string(), json!(d));
        }
        let response = self
            .websocket_send("get_cookies", json!(params))
            .await?;
        let data = response.data(timeout).await?;
        let cookies: Cookies = serde_json::from_value(data)?;
        Ok(cookies)
    }

    /// Call [`Self::get_cookies_with_timeout`] with a ten-second response wait.
    ///
    /// See the timeout version for other behavior and errors.
    pub async fn get_cookies(&self, domain: Option<&str>) -> Result<Cookies> {
        self.get_cookies_with_timeout(domain, Duration::from_secs(10))
            .await
    }
}

