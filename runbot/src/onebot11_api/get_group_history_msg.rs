//! Fetches group message history.
//!
//! The default version waits up to ten seconds for a response and parses each message with
//! [`crate::event::Message::parse`].

use crate::error::Result;
use crate::prelude::BotContext;
use crate::prelude::Message;
use serde_json::json;
use tokio::time::Duration;

impl BotContext {
    /// Requests group message history and waits up to the given time to receive and parse the
    /// response.
    ///
    /// # Arguments
    ///
    /// - `group_id`: target group ID.
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

    /// Calls the corresponding `_with_timeout` method with a ten-second response wait.
    ///
    /// See the timeout variant for other behavior and errors.
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

