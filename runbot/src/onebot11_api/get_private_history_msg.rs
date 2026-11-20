//! Fetches private message history.
//!
//! The default version waits up to ten seconds for a response and parses each message with
//! [`crate::event::Message::parse`].

use crate::error::Result;
use crate::prelude::BotContext;
use crate::prelude::Message;
use serde_json::json;
use tokio::time::Duration;

impl BotContext {
    /// Requests private message history and waits up to the given time to receive and parse the
    /// response.
    ///
    /// # Arguments
    ///
    /// - `user_id`: QQ ID of the target friend.
    /// - `message_seq`: starting message sequence; the meaning of `0` or other specific values
    ///   is defined by the upstream.
    /// - `count`: expected number of messages; the upstream may return fewer.
    /// - `timeout`: response wait duration after the request has been sent.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when the current connection is unavailable
    /// or `retcode` is non-zero; returns [`crate::error::Error::WebSocketError`] when the
    /// WebSocket send fails; returns [`crate::error::Error::TimeoutError`] on timeout; returns
    /// [`crate::error::Error::FieldError`] when the response `data` is not an array or a
    /// message cannot be parsed by [`crate::event::Message::parse`].
    pub async fn get_private_history_msg_with_timeout(
        &self,
        user_id: i64,
        message_seq: i64,
        count: i32,
        timeout: Duration,
    ) -> Result<Vec<Message>> {
        let response = self
            .websocket_send(
                "get_private_history_msg",
                json!({
                    "user_id": user_id,
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

    /// Calls the corresponding `_with_timeout` method with a ten-second response wait.
    ///
    /// See the timeout variant for other behavior and errors.
    pub async fn get_private_history_msg(
        &self,
        user_id: i64,
        message_seq: i64,
        count: i32,
    ) -> Result<Vec<Message>> {
        self.get_private_history_msg_with_timeout(user_id, message_seq, count, Duration::from_secs(10))
            .await
    }
}

