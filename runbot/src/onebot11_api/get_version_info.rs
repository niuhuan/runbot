use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use tokio::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    pub app_name: String,
    pub app_version: String,
    pub protocol_version: String,
}

impl BotContext {
    pub async fn get_version_info_with_timeout(&self, timeout: Duration) -> Result<VersionInfo> {
        let response = self
            .websocket_send("get_version_info", serde_json::Value::Null)
            .await?;
        let data = response.data(timeout).await?;
        let version: VersionInfo = serde_json::from_value(data)?;
        Ok(version)
    }

    pub async fn get_version_info(&self) -> Result<VersionInfo> {
        self.get_version_info_with_timeout(Duration::from_secs(10))
            .await
    }
}

