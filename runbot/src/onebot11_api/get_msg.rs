use crate::error::Result;
use crate::prelude::BotContext;
use crate::prelude::Message;
use serde_json::json;
use tokio::time::Duration;

impl BotContext {
    pub async fn get_msg_with_timeout(
        &self,
        message_id: i64,
        timeout: Duration,
    ) -> Result<Message> {
        let send = json!({
            "message_id": message_id,
        });
        let response = self.websocket_send("get_msg", send).await?;
        let data = response.data(timeout).await?;
        let msg = Message::parse(&data)?;
        Ok(msg)
    }

    pub async fn get_msg(&self, message_id: i64) -> Result<Message> {
        self.get_msg_with_timeout(message_id, Duration::from_secs(3))
            .await
    }
}
