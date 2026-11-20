//! Get group system messages.
//!
//! The default version waits up to ten seconds for the response; invitation and join
//! request contents are returned as raw JSON arrays.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use tokio::time::Duration;

/// Collection of group system messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupSystemMsg {
    /// Group invitation messages; the element structure is defined by the upstream.
    pub invited_requests: Vec<serde_json::Value>,
    /// Group join request messages; the element structure is defined by the upstream.
    pub join_requests: Vec<serde_json::Value>,
}

impl BotContext {
    /// Request group system messages and wait for the response data within the specified
    /// timeout.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when the current connection is
    /// unavailable or `retcode` is non-zero; returns
    /// [`crate::error::Error::WebSocketError`] when the WebSocket send fails; returns
    /// [`crate::error::Error::TimeoutError`] on timeout; returns
    /// [`crate::error::Error::JsonError`] when the response cannot be deserialized into
    /// [`GroupSystemMsg`].
    pub async fn get_group_system_msg_with_timeout(
        &self,
        timeout: Duration,
    ) -> Result<GroupSystemMsg> {
        let response = self
            .websocket_send("get_group_system_msg", serde_json::Value::Null)
            .await?;
        let data = response.data(timeout).await?;
        let msg: GroupSystemMsg = serde_json::from_value(data)?;
        Ok(msg)
    }

    /// Call [`Self::get_group_system_msg_with_timeout`] with a ten-second response wait.
    ///
    /// See the timeout version for other behavior and errors.
    pub async fn get_group_system_msg(&self) -> Result<GroupSystemMsg> {
        self.get_group_system_msg_with_timeout(Duration::from_secs(10))
            .await
    }
}

