//! Fetches basic information for a single group.
//!
//! The request waits up to ten seconds for a response; no partial group information is
//! returned on failure.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use tokio::time::Duration;

/// Basic group information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupInfo {
    /// Group ID.
    pub group_id: i64,
    /// Group name.
    pub group_name: String,
    /// Current member count.
    pub member_count: i64,
    /// Maximum member count, determined by the group configuration.
    pub max_member_count: i64,
}

impl BotContext {
    /// Requests basic information for the specified group and waits up to ten seconds for the
    /// response data.
    ///
    /// A usable connection must already be established before calling this method. The
    /// response wait is fixed at ten seconds and only limits the wait after the send
    /// completes.
    ///
    /// # Arguments
    ///
    /// - `group_id`: target group ID.
    /// - `no_cache`: whether to ask the upstream to bypass its cache; whether this takes
    ///   effect depends on the upstream implementation.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when the current connection is unavailable
    /// or `retcode` is non-zero; returns [`crate::error::Error::WebSocketError`] when the
    /// WebSocket send fails; returns [`crate::error::Error::TimeoutError`] when no response
    /// arrives within ten seconds; returns [`crate::error::Error::JsonError`] when the response
    /// data cannot be deserialized into [`GroupInfo`].
    pub async fn get_group_info(&self, group_id: i64, no_cache: bool) -> Result<GroupInfo> {
        let response = self
            .websocket_send(
                "get_group_info",
                serde_json::json!({
                    "group_id": group_id,
                    "no_cache": no_cache,
                }),
            )
            .await?;
        let response = response.data(Duration::from_secs(10)).await?;
        let group_info: GroupInfo = serde_json::from_value(response)?;
        Ok(group_info)
    }
}
