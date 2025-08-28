use crate::error::Result;
use crate::prelude::BotContext;
use crate::prelude::ForwardMessage;
use serde_json::json;
use tokio::time::Duration;

impl BotContext {
    pub async fn get_forward_msg_with_timeout(
        &self,
        id: &str,
        timeout: Duration,
    ) -> Result<ForwardMessage> {
        let send = json!({
            "id": id,
        });
        let response = self.websocket_send("get_forward_msg", send).await?;
        let data = response.data(timeout).await?;
        let msg = ForwardMessage::parse(&data)?;
        Ok(msg)
    }

    pub async fn get_forward_msg(&self, id: &str) -> Result<ForwardMessage> {
        self.get_forward_msg_with_timeout(id, Duration::from_secs(3))
            .await
    }
}
