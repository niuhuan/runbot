use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use tokio::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnlineModel {
    pub model: String,
    pub model_show: String,
}

impl BotContext {
    pub async fn get_online_model_with_timeout(&self, timeout: Duration) -> Result<OnlineModel> {
        let response = self
            .websocket_send("get_online_model", serde_json::Value::Null)
            .await?;
        let data = response.data(timeout).await?;
        let model: OnlineModel = serde_json::from_value(data)?;
        Ok(model)
    }

    pub async fn get_online_model(&self) -> Result<OnlineModel> {
        self.get_online_model_with_timeout(Duration::from_secs(10))
            .await
    }
}

