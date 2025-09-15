use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use tokio::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivateFileUrl {
    pub url: String,
}

impl BotContext {
    pub async fn get_private_file_url(
        &self,
        file_id: &str,
        user_id: impl Into<Option<i64>>,
    ) -> Result<PrivateFileUrl> {
        let response = self
            .websocket_send(
                "get_private_file_url",
                serde_json::json!({
                    "file_id": file_id,
                    "user_id": user_id.into(),
                }),
            )
            .await?;
        let response = response.data(Duration::from_secs(10)).await?;
        let group_member_info: PrivateFileUrl = serde_json::from_value(response)?;
        Ok(group_member_info)
    }
}
