use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use tokio::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnlineClient {
    pub app_id: i64,
    pub device_name: String,
    pub device_kind: String,
}

impl BotContext {
    pub async fn get_online_clients_with_timeout(
        &self,
        timeout: Duration,
    ) -> Result<Vec<OnlineClient>> {
        let response = self
            .websocket_send("get_online_clients", serde_json::Value::Null)
            .await?;
        let data = response.data(timeout).await?;
        let clients: Vec<OnlineClient> = serde_json::from_value(data)?;
        Ok(clients)
    }

    pub async fn get_online_clients(&self) -> Result<Vec<OnlineClient>> {
        self.get_online_clients_with_timeout(Duration::from_secs(10))
            .await
    }
}

