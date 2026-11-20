//! Fetches extended group information.
//!
//! Compared with [`crate::onebot11_api::get_group_info`], this module returns upstream
//! extension fields such as the group remark, creation time, and group level.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use tokio::time::Duration;

/// Extended group information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupInfoEx {
    /// Group ID.
    pub group_id: i64,
    /// Group name.
    pub group_name: String,
    /// Group remark.
    pub group_memo: String,
    /// Group creation time, Unix timestamp in seconds.
    pub group_create_time: i64,
    /// Group level provided by the upstream.
    pub group_level: i64,
    /// Current member count.
    pub member_count: i64,
    /// Maximum member count.
    pub max_member_count: i64,
}

impl BotContext {
    /// Requests extended group information and waits up to the given time for the response
    /// data.
    ///
    /// `no_cache` indicates whether to ask the upstream to bypass its cache; whether it takes
    /// effect depends on the upstream implementation.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when the current connection is unavailable
    /// or `retcode` is non-zero; returns [`crate::error::Error::WebSocketError`] when the
    /// WebSocket send fails; returns [`crate::error::Error::TimeoutError`] on timeout; returns
    /// [`crate::error::Error::JsonError`] when the response cannot be deserialized into
    /// [`GroupInfoEx`].
    pub async fn get_group_info_ex_with_timeout(
        &self,
        group_id: i64,
        no_cache: bool,
        timeout: Duration,
    ) -> Result<GroupInfoEx> {
        let response = self
            .websocket_send(
                "get_group_info_ex",
                json!({
                    "group_id": group_id,
                    "no_cache": no_cache,
                }),
            )
            .await?;
        let data = response.data(timeout).await?;
        let info: GroupInfoEx = serde_json::from_value(data)?;
        Ok(info)
    }

    /// Calls [`Self::get_group_info_ex_with_timeout`] with a ten-second response wait.
    ///
    /// See the timeout variant for other behavior and errors.
    pub async fn get_group_info_ex(&self, group_id: i64, no_cache: bool) -> Result<GroupInfoEx> {
        self.get_group_info_ex_with_timeout(group_id, no_cache, Duration::from_secs(10))
            .await
    }
}

