use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use tokio::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RKey {
    pub rkey: String,
}

impl BotContext {
    pub async fn get_rkey_with_timeout(
        &self,
        url: &str,
        timeout: Duration,
    ) -> Result<RKey> {
        let response = self
            .websocket_send(
                "get_rkey",
                json!({
                    "url": url,
                }),
            )
            .await?;
        let data = response.data(timeout).await?;
        let rkey: RKey = serde_json::from_value(data)?;
        Ok(rkey)
    }

    pub async fn get_rkey(&self, url: &str) -> Result<RKey> {
        self.get_rkey_with_timeout(url, Duration::from_secs(10))
            .await
    }
}

