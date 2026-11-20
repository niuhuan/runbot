//! Fetches the profile of a single group member.
//!
//! The request waits up to ten seconds for a response; the meaning and availability of the
//! profile fields depend on the upstream implementation.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use tokio::time::Duration;

/// Group member profile.
///
/// Except for [`Self::user_id`] and [`Self::nickname`], missing fields use the default value
/// of their respective types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMemberInfo {
    /// QQ ID of the member.
    pub user_id: i64,
    /// Member nickname.
    pub nickname: String,
    /// Group card; empty string when missing.
    #[serde(default)]
    pub card: String,
    /// Combined display value of the group card or nickname; empty string when missing; the
    /// exact value is determined by the upstream.
    #[serde(default)]
    pub card_or_nickname: String,
    /// Gender string provided by the upstream; empty string when missing.
    #[serde(default)]
    pub sex: String,
    /// Age provided by the upstream; `0` when missing.
    #[serde(default)]
    pub age: i64,
    /// Group level provided by the upstream; `0` when missing.
    #[serde(default)]
    pub level: i64,
    /// Group role, such as `owner`, `admin`, or `member`; empty string when missing.
    #[serde(default)]
    pub role: String,
    /// Unfriendly status flag set by the upstream; `false` when missing; the meaning is
    /// determined by the upstream.
    #[serde(default)]
    pub unfriendly: bool,
    /// Exclusive title; empty string when there is none or the field is missing.
    #[serde(default)]
    pub title: String,
    /// Title expiration time, conventionally a Unix timestamp in seconds in OneBot; `0` when
    /// missing or non-expiring.
    #[serde(default)]
    pub title_expire_time: i64,
    /// Whether the group card can be changed; `false` when missing.
    #[serde(default)]
    pub card_changeable: bool,
    /// Region information provided by the upstream; empty string when missing.
    #[serde(default)]
    pub area: String,
    /// Owning group ID; `0` when missing.
    #[serde(default)]
    pub group_id: i64,
    /// Join time, Unix timestamp in seconds; `0` when missing.
    #[serde(default)]
    pub join_time: i64,
    /// Last message time, Unix timestamp in seconds; `0` when missing.
    #[serde(default)]
    pub last_sent_time: i64,
}

impl BotContext {
    /// Requests the profile of the specified group member and waits up to ten seconds for the
    /// response data.
    ///
    /// # Arguments
    ///
    /// - `group_id`: target group ID.
    /// - `user_id`: QQ ID of the target member.
    /// - `no_cache`: whether to ask the upstream to bypass its cache; whether this takes
    ///   effect depends on the upstream implementation.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when the current connection is unavailable
    /// or `retcode` is non-zero; returns [`crate::error::Error::WebSocketError`] when the
    /// WebSocket send fails; returns [`crate::error::Error::TimeoutError`] when no response
    /// arrives within ten seconds; returns [`crate::error::Error::JsonError`] when the response
    /// data cannot be deserialized into [`GroupMemberInfo`].
    pub async fn get_group_member_info(
        &self,
        group_id: i64,
        user_id: i64,
        no_cache: bool,
    ) -> Result<GroupMemberInfo> {
        let response = self
            .websocket_send(
                "get_group_member_info",
                serde_json::json!({
                    "group_id": group_id,
                    "user_id": user_id,
                    "no_cache": no_cache,
                }),
            )
            .await?;
        let response = response.data(Duration::from_secs(10)).await?;
        let group_member_info: GroupMemberInfo = serde_json::from_value(response)?;
        Ok(group_member_info)
    }
}
