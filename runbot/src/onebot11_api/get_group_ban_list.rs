//! Get the group mute list.
//!
//! The default version waits up to ten seconds for the response; the unit of the remaining
//! mute duration is defined by the upstream.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use tokio::time::Duration;

/// A group mute record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupBanInfo {
    /// QQ number of the muted member.
    pub user_id: i64,
    /// Member nickname.
    pub nickname: String,
    /// Remaining mute duration, usually in seconds; the exact unit is defined by the
    /// upstream.
    pub time_left: i64,
}

impl BotContext {
    /// Request the group mute list and wait for the response data within the specified
    /// timeout.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when the current connection is
    /// unavailable or `retcode` is non-zero; returns
    /// [`crate::error::Error::WebSocketError`] when the WebSocket send fails; returns
    /// [`crate::error::Error::TimeoutError`] on timeout; returns
    /// [`crate::error::Error::JsonError`] when the response cannot be deserialized into a
    /// list of [`GroupBanInfo`].
    pub async fn get_group_ban_list_with_timeout(
        &self,
        group_id: i64,
        timeout: Duration,
    ) -> Result<Vec<GroupBanInfo>> {
        let response = self
            .websocket_send(
                "get_group_ban_list",
                json!({
                    "group_id": group_id,
                }),
            )
            .await?;
        let data = response.data(timeout).await?;
        let bans: Vec<GroupBanInfo> = serde_json::from_value(data)?;
        Ok(bans)
    }

    /// Call [`Self::get_group_ban_list_with_timeout`] with a ten-second response wait.
    ///
    /// See the timeout version for other behavior and errors.
    pub async fn get_group_ban_list(&self, group_id: i64) -> Result<Vec<GroupBanInfo>> {
        self.get_group_ban_list_with_timeout(group_id, Duration::from_secs(10))
            .await
    }
}

