//! Get filtered group system messages.
//!
//! The default version waits up to ten seconds for the response; the filter type strings
//! are defined by the upstream.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use tokio::time::Duration;

/// Filtered group system message configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupFilterSystemMsg {
    /// Group number.
    pub group_id: i64,
    /// List of filter types; values are defined by the upstream.
    pub filter_type: Vec<String>,
}

impl BotContext {
    /// Request filtered group system messages and wait for the response data within the
    /// specified timeout.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when the current connection is
    /// unavailable or `retcode` is non-zero; returns
    /// [`crate::error::Error::WebSocketError`] when the WebSocket send fails; returns
    /// [`crate::error::Error::TimeoutError`] on timeout; returns
    /// [`crate::error::Error::JsonError`] when the response cannot be deserialized into
    /// [`GroupFilterSystemMsg`].
    pub async fn get_group_filter_system_msg_with_timeout(
        &self,
        group_id: i64,
        timeout: Duration,
    ) -> Result<GroupFilterSystemMsg> {
        let response = self
            .websocket_send(
                "get_group_filter_system_msg",
                json!({
                    "group_id": group_id,
                }),
            )
            .await?;
        let data = response.data(timeout).await?;
        let msg: GroupFilterSystemMsg = serde_json::from_value(data)?;
        Ok(msg)
    }

    /// Call [`Self::get_group_filter_system_msg_with_timeout`] with a ten-second response
    /// wait.
    ///
    /// See the timeout version for other behavior and errors.
    pub async fn get_group_filter_system_msg(&self, group_id: i64) -> Result<GroupFilterSystemMsg> {
        self.get_group_filter_system_msg_with_timeout(group_id, Duration::from_secs(10))
            .await
    }
}

