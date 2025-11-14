use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use tokio::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsrfToken {
    pub token: i32,
}

impl BotContext {
    pub async fn get_csrf_token_with_timeout(&self, timeout: Duration) -> Result<CsrfToken> {
        let response = self
            .websocket_send("get_csrf_token", serde_json::Value::Null)
            .await?;
        let data = response.data(timeout).await?;
        let token: CsrfToken = serde_json::from_value(data)?;
        Ok(token)
    }

    pub async fn get_csrf_token(&self) -> Result<CsrfToken> {
        self.get_csrf_token_with_timeout(Duration::from_secs(10))
            .await
    }
}

