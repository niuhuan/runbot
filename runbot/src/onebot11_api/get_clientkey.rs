//! Gets the client key (upstream extension action `get_clientkey`).
//!
//! The default version waits up to ten seconds for a response; the `_with_timeout` version
//! allows a custom response wait duration.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use tokio::time::Duration;

/// Response data for `get_clientkey`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientKey {
    /// The client key returned by the upstream.
    ///
    /// This value is sensitive credential data and should not be written to logs, examples,
    /// or public output. Its validity period and the APIs it applies to are determined by the
    /// upstream implementation. The field is required during deserialization.
    pub client_key: String,
}

impl BotContext {
    /// Requests the client key and waits for the response data within the specified time.
    ///
    /// An available connection should exist before calling this method. `timeout` limits the
    /// response wait after the request has been sent; it does not include serialization,
    /// lock acquisition, or the WebSocket write. This method does not retry.
    ///
    /// # Errors
    ///
    /// - Returns [`crate::error::Error::StateError`] when no connection is available or the
    ///   response channel is closed.
    /// - Returns [`crate::error::Error::WebSocketError`] when the WebSocket send fails.
    /// - Returns [`crate::error::Error::TimeoutError`] when no response arrives within
    ///   `timeout`.
    /// - Returns [`crate::error::Error::StateError`] when the upstream response has a
    ///   non-zero `retcode`.
    /// - Returns [`crate::error::Error::JsonError`] when the response data cannot be
    ///   deserialized into [`ClientKey`].
    ///
    /// # Cancellation
    ///
    /// Dropping the pending future stops the local wait and releases the response
    /// registration; the request already sent is not revoked from the upstream.
    pub async fn get_clientkey_with_timeout(&self, timeout: Duration) -> Result<ClientKey> {
        let response = self
            .websocket_send("get_clientkey", serde_json::Value::Null)
            .await?;
        let data = response.data(timeout).await?;
        let key: ClientKey = serde_json::from_value(data)?;
        Ok(key)
    }

    /// Calls [`Self::get_clientkey_with_timeout`] with a ten-second response wait.
    ///
    /// Connection, send, timeout, `retcode`, and deserialization errors are the same as in
    /// the version with a timeout.
    pub async fn get_clientkey(&self) -> Result<ClientKey> {
        self.get_clientkey_with_timeout(Duration::from_secs(10))
            .await
    }
}

