use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use tokio::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub cookies: String,
    pub csrf_token: i32,
}

impl BotContext {
    pub async fn get_credentials_with_timeout(
        &self,
        domain: Option<&str>,
        timeout: Duration,
    ) -> Result<Credentials> {
        let mut params = serde_json::Map::new();
        if let Some(d) = domain {
            params.insert("domain".to_string(), json!(d));
        }
        let response = self
            .websocket_send("get_credentials", json!(params))
            .await?;
        let data = response.data(timeout).await?;
        let creds: Credentials = serde_json::from_value(data)?;
        Ok(creds)
    }

    pub async fn get_credentials(&self, domain: Option<&str>) -> Result<Credentials> {
        self.get_credentials_with_timeout(domain, Duration::from_secs(10))
            .await
    }
}

