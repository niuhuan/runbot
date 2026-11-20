//! Wraps the OneBot action `get_recent_contact`.
//!
//! The method sends a query request, waits for the response with the matching
//! echo, checks `retcode`, and deserializes `data` into a list of
//! [`RecentContact`] values.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use tokio::time::Duration;

/// One recent conversation entry returned by `get_recent_contact`.
///
/// `user_id` and `group_id` are `Option`: a missing field or an explicit `null`
/// both become `None`. `time` is required, so a missing field or `null` causes
/// deserialization to fail. This library does not validate how the private-chat and
/// group-chat fields are combined.
///
/// # Notes
///
/// The field names are fixed by the current implementation. Other upstream
/// extensions may use different keys for `get_recent_contact`, in which case
/// deserialization fails with [`crate::error::Error::JsonError`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentContact {
    /// QQ number of the peer in a private conversation; `None` for a group
    /// conversation, when the upstream omits it, or when the field is missing.
    pub user_id: Option<i64>,
    /// Group number for a group conversation; `None` for a private conversation,
    /// when the upstream omits it, or when the field is missing.
    pub group_id: Option<i64>,
    /// Conversation time value supplied by the upstream; this library does not
    /// interpret its unit or convert time zones.
    pub time: i64,
}

impl BotContext {
    /// Requests recent conversation entries and waits up to `timeout` for the
    /// response data.
    ///
    /// `timeout` limits only the response wait after the request has been
    /// written to the connection. It does not cover JSON serialization, internal
    /// lock acquisition, or the WebSocket write. The order of the returned list
    /// is determined by the upstream; this method does not sort it.
    ///
    /// # Arguments
    ///
    /// - `timeout`: how long to wait for the response with the matching echo.
    ///
    /// # Errors
    ///
    /// The wait follows the semantics of [`crate::bot_context::EchoAsyncResponse::data`]: returns
    /// [`crate::error::Error::StateError`] when the current connection is
    /// unavailable (not connected, disconnected, or closed) or the response
    /// channel closes early; [`crate::error::Error::WebSocketError`] when the
    /// WebSocket write fails; [`crate::error::Error::TimeoutError`] when no
    /// response arrives within `timeout`; and
    /// [`crate::error::Error::StateError`] when the upstream `retcode` is
    /// non-zero, with the error text taken from the response `message`. In
    /// addition, returns [`crate::error::Error::JsonError`] when `data` cannot
    /// be deserialized into `Vec<RecentContact>`.
    ///
    /// # Cancellation
    ///
    /// Dropping the pending future stops the local wait and removes the matching
    /// echo registration; the request has already been written to the
    /// connection, so cancellation does not withdraw it from the upstream.
    pub async fn get_recent_contact_with_timeout(
        &self,
        timeout: Duration,
    ) -> Result<Vec<RecentContact>> {
        let response = self
            .websocket_send("get_recent_contact", serde_json::Value::Null)
            .await?;
        let data = response.data(timeout).await?;
        let contacts: Vec<RecentContact> = serde_json::from_value(data)?;
        Ok(contacts)
    }

    /// Gets recent conversation entries using a 10-second response wait.
    ///
    /// Except for the fixed wait duration, the arguments, return value, and
    /// error semantics are the same as
    /// [`Self::get_recent_contact_with_timeout`].
    pub async fn get_recent_contact(&self) -> Result<Vec<RecentContact>> {
        self.get_recent_contact_with_timeout(Duration::from_secs(10))
            .await
    }
}

