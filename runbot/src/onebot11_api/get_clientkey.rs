use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use tokio::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientKey {
    pub client_key: String,
}

impl BotContext {
    pub async fn get_clientkey_with_timeout(&self, timeout: Duration) -> Result<ClientKey> {
        let response = self
            .websocket_send("get_clientkey", serde_json::Value::Null)
            .await?;
        let data = response.data(timeout).await?;
        let key: ClientKey = serde_json::from_value(data)?;
        Ok(key)
    }

    pub async fn get_clientkey(&self) -> Result<ClientKey> {
        self.get_clientkey_with_timeout(Duration::from_secs(10))
            .await
    }
}

