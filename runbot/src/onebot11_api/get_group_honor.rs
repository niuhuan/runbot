//! Fetches group honor information.
//!
//! The request returns the list of honor members and the current talkative member within the
//! specified response wait time.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use tokio::time::Duration;

/// A group honor member.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupHonorMember {
    /// QQ ID of the member.
    pub user_id: i64,
    /// Member nickname.
    pub nickname: String,
    /// Avatar URL provided by the upstream.
    pub avatar: String,
    /// Honor description provided by the upstream.
    pub description: String,
}

/// Group honor information.
///
/// The lists returned by different upstreams may be empty; `current_talkative` is `None` when
/// missing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupHonor {
    /// Group ID.
    pub group_id: i64,
    /// Current talkative member ("dragon king"); `None` when the upstream does not provide it.
    pub current_talkative: Option<GroupHonorMember>,
    /// List of talkative members.
    pub talkative_list: Vec<GroupHonorMember>,
    /// List of group chat fire honor members.
    pub performer_list: Vec<GroupHonorMember>,
    /// List of group chat legend honor members.
    pub legend_list: Vec<GroupHonorMember>,
    /// List of group chat rising star honor members.
    pub strong_newbie_list: Vec<GroupHonorMember>,
    /// List of group chat blazing flame honor members.
    pub emotion_list: Vec<GroupHonorMember>,
}

impl BotContext {
    /// Requests group honor information and waits up to the given time for the response data.
    ///
    /// # Arguments
    ///
    /// - `group_id`: target group ID.
    /// - `honor_type`: honor type string, sent to the upstream as is; common values include
    ///   `talkative`, `performer`, `legend`, `strong_newbie`, and `emotion`; the exact set
    ///   supported is determined by the upstream. This method does not validate the value
    ///   locally.
    /// - `timeout`: response wait duration after the request has been sent.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when the current connection is unavailable
    /// or `retcode` is non-zero; returns [`crate::error::Error::WebSocketError`] when the
    /// WebSocket send fails; returns [`crate::error::Error::TimeoutError`] on timeout; returns
    /// [`crate::error::Error::JsonError`] when the response cannot be deserialized into
    /// [`GroupHonor`].
    pub async fn get_group_honor_with_timeout(
        &self,
        group_id: i64,
        honor_type: &str,
        timeout: Duration,
    ) -> Result<GroupHonor> {
        let response = self
            .websocket_send(
                "get_group_honor",
                json!({
                    "group_id": group_id,
                    "type": honor_type,
                }),
            )
            .await?;
        let data = response.data(timeout).await?;
        let honor: GroupHonor = serde_json::from_value(data)?;
        Ok(honor)
    }

    /// Calls [`Self::get_group_honor_with_timeout`] with a ten-second response wait.
    ///
    /// See the timeout variant for other behavior and errors.
    pub async fn get_group_honor(&self, group_id: i64, honor_type: &str) -> Result<GroupHonor> {
        self.get_group_honor_with_timeout(group_id, honor_type, Duration::from_secs(10))
            .await
    }
}

