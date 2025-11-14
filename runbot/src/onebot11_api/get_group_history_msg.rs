use crate::error::Result;
use crate::prelude::BotContext;
use crate::prelude::Message;
use serde_json::json;
use tokio::time::Duration;

impl BotContext {
    pub async fn get_group_history_msg_with_timeout(
        &self,
        group_id: i64,
        message_seq: i64,
        count: i32,
        timeout: Duration,
    ) -> Result<Vec<Message>> {
        let response = self
            .websocket_send(
                "get_group_history_msg",
                json!({
                    "group_id": group_id,
                    "message_seq": message_seq,
                    "count": count,
                }),
            )
            .await?;
        let data = response.data(timeout).await?;
        let messages_array = data
            .as_array()
            .ok_or(crate::error::Error::FieldError("expected array".to_string()))?;
        let mut messages = Vec::new();
        for msg_value in messages_array {
            messages.push(Message::parse(msg_value)?);
        }
        Ok(messages)
    }

    pub async fn get_group_history_msg(
        &self,
        group_id: i64,
        message_seq: i64,
        count: i32,
    ) -> Result<Vec<Message>> {
        self.get_group_history_msg_with_timeout(group_id, message_seq, count, Duration::from_secs(10))
            .await
    }
}

