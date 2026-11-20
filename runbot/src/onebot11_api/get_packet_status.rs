//! Query the processing status of a custom packet.
//!
//! The default version waits up to ten seconds for the response; `packet_id` is assigned by
//! the upstream when the packet is sent.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use tokio::time::Duration;

/// Packet processing status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacketStatus {
    /// Status text returned by the upstream; the values are implementation-defined.
    pub status: String,
}

impl BotContext {
    /// Request the processing status of the specified packet and wait for the response data
    /// within the specified timeout.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when the current connection is
    /// unavailable or `retcode` is non-zero; returns
    /// [`crate::error::Error::WebSocketError`] when the WebSocket send fails; returns
    /// [`crate::error::Error::TimeoutError`] on timeout; returns
    /// [`crate::error::Error::JsonError`] when the response cannot be deserialized into
    /// [`PacketStatus`].
    pub async fn get_packet_status_with_timeout(
        &self,
        packet_id: &str,
        timeout: Duration,
    ) -> Result<PacketStatus> {
        let response = self
            .websocket_send(
                "get_packet_status",
                json!({
                    "packet_id": packet_id,
                }),
            )
            .await?;
        let data = response.data(timeout).await?;
        let status: PacketStatus = serde_json::from_value(data)?;
        Ok(status)
    }

    /// Call [`Self::get_packet_status_with_timeout`] with a ten-second response wait.
    ///
    /// See the timeout version for other behavior and errors.
    pub async fn get_packet_status(&self, packet_id: &str) -> Result<PacketStatus> {
        self.get_packet_status_with_timeout(packet_id, Duration::from_secs(10))
            .await
    }
}

