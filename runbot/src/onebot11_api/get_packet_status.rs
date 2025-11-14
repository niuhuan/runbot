use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use tokio::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacketStatus {
    pub status: String,
}

impl BotContext {
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

    pub async fn get_packet_status(&self, packet_id: &str) -> Result<PacketStatus> {
        self.get_packet_status_with_timeout(packet_id, Duration::from_secs(10))
            .await
    }
}

