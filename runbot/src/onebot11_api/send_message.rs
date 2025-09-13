use crate::error::{Error, Result};
use crate::prelude::BotContext;
use crate::prelude::EchoAsyncResponse;
use crate::prelude::MessageType;
use crate::prelude::SendMessage;
use crate::re_export::serde_json;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use tokio::time::Duration;

pub struct SendMessageAsyncResponse(EchoAsyncResponse);

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub struct SendMessageResponse {
    pub message_id: i64,
}

impl SendMessageAsyncResponse {
    pub async fn wait_response_with_timeout(
        self,
        timeout: Duration,
    ) -> Result<SendMessageResponse> {
        Ok(serde_json::from_value(self.0.data(timeout).await?)?)
    }

    pub async fn wait_response(self) -> Result<SendMessageResponse> {
        self.wait_response_with_timeout(Duration::from_secs(10))
            .await
    }
}

impl BotContext {
    pub async fn send_private_message(
        &self,
        user_id: i64,
        message: impl SendMessage,
    ) -> Result<SendMessageAsyncResponse> {
        let msg = json!(
            {
                "user_id": user_id,
                "message": message.chain(),
            }
        );
        self.websocket_send("send_private_msg", msg)
            .await
            .map(|r| SendMessageAsyncResponse(r))
    }

    pub async fn send_group_message(
        &self,
        group_id: i64,
        message: impl SendMessage,
    ) -> Result<SendMessageAsyncResponse> {
        let msg = json!(
            {
                "group_id": group_id,
                "message": message.chain(),
            }
        );
        self.websocket_send("send_group_msg", msg)
            .await
            .map(|r| SendMessageAsyncResponse(r))
    }

    pub async fn send_message(
        &self,
        message_type: MessageType,
        target_id: i64,
        message: impl SendMessage,
    ) -> Result<SendMessageAsyncResponse> {
        match message_type {
            MessageType::Private => self.send_private_message(target_id, message).await,
            MessageType::Group => self.send_group_message(target_id, message).await,
            _ => Err(Error::FieldError("unknown message_type".to_string())),
        }
    }
}
