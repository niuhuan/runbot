use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use tokio::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInfo {
    pub user_id: i64,
    pub nickname: String,
    pub sex: String,
    pub age: i64,
}

impl BotContext {
    pub async fn get_account_info_with_timeout(&self, timeout: Duration) -> Result<AccountInfo> {
        let response = self
            .websocket_send("get_account_info", serde_json::Value::Null)
            .await?;
        let data = response.data(timeout).await?;
        let info: AccountInfo = serde_json::from_value(data)?;
        Ok(info)
    }

    pub async fn get_account_info(&self) -> Result<AccountInfo> {
        self.get_account_info_with_timeout(Duration::from_secs(10))
            .await
    }
}

