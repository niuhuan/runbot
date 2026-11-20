//! Fetches the contents of a merged forward message.
//!
//! The response data is parsed by [`crate::event::ForwardMessage::parse`]; the default
//! response wait is three seconds.

use crate::error::Result;
use crate::prelude::BotContext;
use crate::prelude::ForwardMessage;
use serde_json::json;
use tokio::time::Duration;

impl BotContext {
    /// Requests the contents of the specified merged forward message and waits up to the given
    /// time to receive and parse the response.
    ///
    /// # Arguments
    ///
    /// - `id`: merged forward message ID, usually from [`crate::event::MessageForward::id`].
    /// - `timeout`: response wait duration after the request has been sent; does not include
    ///   the send phase.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when the current connection is unavailable
    /// or `retcode` is non-zero; returns [`crate::error::Error::WebSocketError`] when the
    /// WebSocket send fails; returns [`crate::error::Error::TimeoutError`] on timeout; returns
    /// [`crate::error::Error::FieldError`] when the response data does not satisfy the field
    /// requirements of [`crate::event::ForwardMessage::parse`].
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

    /// Calls [`Self::get_forward_msg_with_timeout`] with a three-second response wait.
    ///
    /// See the timeout variant for other behavior and errors.
    pub async fn get_forward_msg(&self, id: &str) -> Result<ForwardMessage> {
        self.get_forward_msg_with_timeout(id, Duration::from_secs(3))
            .await
    }
}
