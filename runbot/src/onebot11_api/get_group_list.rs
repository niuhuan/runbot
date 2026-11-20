//! Fetches the list of groups the current account has joined.
//!
//! The request waits up to ten seconds for a response; the list order is determined by the
//! upstream response, and this module does not reorder it.

use crate::error::Result;
use crate::onebot11_api::get_group_info::GroupInfo;
use crate::prelude::BotContext;
use serde_json::json;
use tokio::time::Duration;

impl BotContext {
    /// Requests the group list and waits up to ten seconds for the response data.
    ///
    /// # Arguments
    ///
    /// - `no_cache`: whether to ask the upstream to bypass its cache; whether this takes
    ///   effect depends on the upstream implementation.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when the current connection is unavailable
    /// or `retcode` is non-zero; returns [`crate::error::Error::WebSocketError`] when the
    /// WebSocket send fails; returns [`crate::error::Error::TimeoutError`] when no response
    /// arrives within ten seconds; returns [`crate::error::Error::JsonError`] when the response
    /// data cannot be deserialized into a list of [`GroupInfo`].
    pub async fn get_group_list(&self, no_cache: bool) -> Result<Vec<GroupInfo>> {
        let response = self
            .websocket_send("get_group_list", json!({ "no_cache": no_cache }))
            .await?;
        let response = response.data(Duration::from_secs(10)).await?;
        let group_list: Vec<GroupInfo> = serde_json::from_value(response)?;
        Ok(group_list)
    }
}
