use crate::error::Result;
use crate::onebot11_api::get_group_info::GroupInfo;
use crate::prelude::BotContext;
use tokio::time::Duration;

impl BotContext {
    pub async fn get_group_list(&self) -> Result<Vec<GroupInfo>> {
        let response = self
            .websocket_send("get_group_list", serde_json::Value::Null)
            .await?;
        let response = response.data(Duration::from_secs(10)).await?;
        let group_list: Vec<GroupInfo> = serde_json::from_value(response)?;
        Ok(group_list)
    }
}
