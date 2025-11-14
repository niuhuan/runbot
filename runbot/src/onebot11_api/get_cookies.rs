use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use tokio::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cookies {
    pub cookies: String,
}

impl BotContext {
    pub async fn get_cookies_with_timeout(
        &self,
        domain: Option<&str>,
        timeout: Duration,
    ) -> Result<Cookies> {
        let mut params = serde_json::Map::new();
        if let Some(d) = domain {
            params.insert("domain".to_string(), json!(d));
        }
        let response = self
            .websocket_send("get_cookies", json!(params))
            .await?;
        let data = response.data(timeout).await?;
        let cookies: Cookies = serde_json::from_value(data)?;
        Ok(cookies)
    }

    pub async fn get_cookies(&self, domain: Option<&str>) -> Result<Cookies> {
        self.get_cookies_with_timeout(domain, Duration::from_secs(10))
            .await
    }
}

