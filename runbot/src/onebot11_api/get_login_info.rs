use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use tokio::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginInfo {
    pub user_id: i64,
    pub nickname: String,
}

impl BotContext {
    pub async fn get_login_info_with_timeout(&self, timeout: Duration) -> Result<LoginInfo> {
        let response = self
            .websocket_send("get_login_info", serde_json::Value::Null)
            .await?;
        let data = response.data(timeout).await?;
        let login_info: LoginInfo = serde_json::from_value(data)?;
        Ok(login_info)
    }

    pub async fn get_login_info(&self) -> Result<LoginInfo> {
        self.get_login_info_with_timeout(Duration::from_secs(10))
            .await
    }
}

