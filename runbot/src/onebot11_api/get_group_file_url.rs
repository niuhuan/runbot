// get_group_file_url

use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use tokio::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupFileUrl {
    pub url: String,
}

impl BotContext {
    pub async fn get_group_file_url(&self, group_id: i64, file_id: &str) -> Result<GroupFileUrl> {
        let response = self
            .websocket_send(
                "get_group_file_url",
                serde_json::json!({
                    "group_id": group_id,
                    "file_id": file_id,
                }),
            )
            .await?;
        let response = response.data(Duration::from_secs(10)).await?;
        let group_member_info: GroupFileUrl = serde_json::from_value(response)?;
        Ok(group_member_info)
    }
}
