use crate::error::Result;
use crate::onebot11_api::get_group_member_info::GroupMemberInfo;
use crate::prelude::BotContext;
use tokio::time::Duration;

impl BotContext {
    pub async fn get_group_member_list(
        &self,
        group_id: i64,
        no_cache: bool,
    ) -> Result<Vec<GroupMemberInfo>> {
        let response = self
            .websocket_send(
                "get_group_member_list",
                serde_json::json!({
                    "group_id": group_id,
                    "no_cache": no_cache,
                }),
            )
            .await?;
        let response = response.data(Duration::from_secs(10)).await?;
        let group_member_list: Vec<GroupMemberInfo> = serde_json::from_value(response)?;
        Ok(group_member_list)
    }
}
