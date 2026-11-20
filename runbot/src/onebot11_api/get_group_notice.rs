//! Get group notices.
//!
//! The default version waits up to ten seconds for the response; the notice body is
//! returned as a raw JSON value.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use tokio::time::Duration;

/// A group notice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupNotice {
    /// QQ number of the publisher.
    pub sender_id: i64,
    /// Publication time as a Unix timestamp in seconds.
    pub publish_time: i64,
    /// Notice body and raw structures such as images, as defined by the upstream.
    pub message: serde_json::Value,
}

impl BotContext {
    /// Request the group notice list and wait for the response data within the specified
    /// timeout.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when the current connection is
    /// unavailable or `retcode` is non-zero; returns
    /// [`crate::error::Error::WebSocketError`] when the WebSocket send fails; returns
    /// [`crate::error::Error::TimeoutError`] on timeout; returns
    /// [`crate::error::Error::JsonError`] when the response cannot be deserialized into a
    /// list of [`GroupNotice`].
    pub async fn get_group_notice_with_timeout(
        &self,
        group_id: i64,
        timeout: Duration,
    ) -> Result<Vec<GroupNotice>> {
        let response = self
            .websocket_send(
                "get_group_notice",
                json!({
                    "group_id": group_id,
                }),
            )
            .await?;
        let data = response.data(timeout).await?;
        let notices: Vec<GroupNotice> = serde_json::from_value(data)?;
        Ok(notices)
    }

    /// Call [`Self::get_group_notice_with_timeout`] with a ten-second response wait.
    ///
    /// See the timeout version for other behavior and errors.
    pub async fn get_group_notice(&self, group_id: i64) -> Result<Vec<GroupNotice>> {
        self.get_group_notice_with_timeout(group_id, Duration::from_secs(10))
            .await
    }
}

