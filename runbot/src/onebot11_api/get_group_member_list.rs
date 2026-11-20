//! Fetches the group member list.
//!
//! The request waits up to ten seconds for a response; the list order is determined by the
//! upstream response, and this module does not reorder it.

use crate::error::Result;
use crate::onebot11_api::get_group_member_info::GroupMemberInfo;
use crate::prelude::BotContext;
use tokio::time::Duration;

impl BotContext {
    /// Requests the member list of the specified group and waits up to ten seconds for the
    /// response data.
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
    /// data cannot be deserialized into a list of [`GroupMemberInfo`].
    pub async fn get_group_member_list(
        &self,
        group_id: i64,
        no_cache: bool,
    ) -> Result<Vec<GroupMemberInfo>> {
        let response = self
            .websocket_send(
                "get_group_member_list",
                serde_json::json!({
                    "group_id": group_id,
                    "no_cache": no_cache,
                }),
            )
            .await?;
        let response = response.data(Duration::from_secs(10)).await?;
        let group_member_list: Vec<GroupMemberInfo> = serde_json::from_value(response)?;
        Ok(group_member_list)
    }
}
