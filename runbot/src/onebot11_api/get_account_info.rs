//! Gets information about the current logged-in account (upstream extension action
//! `get_account_info`).
//!
//! The default version waits up to ten seconds for a response; the `_with_timeout` version
//! allows a custom response wait duration.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use tokio::time::Duration;

/// Information about the current logged-in account.
///
/// All fields are required during deserialization; a missing field or a type mismatch results
/// in [`crate::error::Error::JsonError`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInfo {
    /// Account QQ ID.
    pub user_id: i64,
    /// Account nickname.
    pub nickname: String,
    /// Gender string; the concrete values are determined by the upstream implementation.
    pub sex: String,
    /// Age in years, provided by the upstream.
    pub age: i64,
}

impl BotContext {
    /// Requests information about the current logged-in account and waits for the response
    /// data within the specified time.
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
    ///   deserialized into [`AccountInfo`].
    ///
    /// # Cancellation
    ///
    /// Dropping the pending future stops the local wait and releases the response
    /// registration; the request already sent is not revoked from the upstream.
    pub async fn get_account_info_with_timeout(&self, timeout: Duration) -> Result<AccountInfo> {
        let response = self
            .websocket_send("get_account_info", serde_json::Value::Null)
            .await?;
        let data = response.data(timeout).await?;
        let info: AccountInfo = serde_json::from_value(data)?;
        Ok(info)
    }

    /// Calls [`Self::get_account_info_with_timeout`] with a ten-second response wait.
    ///
    /// Connection, send, timeout, `retcode`, and deserialization errors are the same as in
    /// the version with a timeout.
    pub async fn get_account_info(&self) -> Result<AccountInfo> {
        self.get_account_info_with_timeout(Duration::from_secs(10))
            .await
    }
}

