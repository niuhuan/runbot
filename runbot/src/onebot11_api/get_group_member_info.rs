use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use tokio::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMemberInfo {
    pub user_id: i64,
    pub nickname: String,
    #[serde(default)]
    pub card: String,
    #[serde(default)]
    pub sex: String,
    #[serde(default)]
    pub age: i64,
    #[serde(default)]
    pub level: i64,
    #[serde(default)]
    pub role: String,
    #[serde(default)]
    pub unfriendly: bool,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub title_expire_time: i64,
    #[serde(default)]
    pub card_changeable: bool,
}

impl BotContext {
    pub async fn get_group_member_info(
        &self,
        group_id: i64,
        user_id: i64,
        no_cache: bool,
    ) -> Result<GroupMemberInfo> {
        let response = self
            .websocket_send(
                "get_group_member_info",
                serde_json::json!({
                    "group_id": group_id,
                    "user_id": user_id,
                    "no_cache": no_cache,
                }),
            )
            .await?;
        let response = response.data(Duration::from_secs(10)).await?;
        let group_member_info: GroupMemberInfo = serde_json::from_value(response)?;
        Ok(group_member_info)
    }
}
