//! Fetches message details by message ID.
//!
//! The response data is parsed by [`crate::event::Message::parse`]; the default response wait
//! is three seconds.

use crate::error::Result;
use crate::prelude::BotContext;
use crate::prelude::Message;
use serde_json::json;
use tokio::time::Duration;

impl BotContext {
    /// Requests details for the specified message and waits up to the given time to receive and
    /// parse the response.
    ///
    /// # Arguments
    ///
    /// - `message_id`: target message ID.
    /// - `timeout`: response wait duration after the request has been sent; does not include
    ///   the send phase.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when the current connection is unavailable
    /// or `retcode` is non-zero; returns [`crate::error::Error::WebSocketError`] when the
    /// WebSocket send fails; returns [`crate::error::Error::TimeoutError`] on timeout; returns
    /// [`crate::error::Error::FieldError`] when the response data does not satisfy the field
    /// requirements of [`crate::event::Message::parse`].
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

    /// Calls [`Self::get_msg_with_timeout`] with a three-second response wait.
    ///
    /// See the timeout variant for other behavior and errors.
    pub async fn get_msg(&self, message_id: i64) -> Result<Message> {
        self.get_msg_with_timeout(message_id, Duration::from_secs(3))
            .await
    }
}
