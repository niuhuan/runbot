//! Fetches mini-app card information.
//!
//! Requests that the upstream parse the mini-app card identified by a message ID and waits up
//! to the given time for the result.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use tokio::time::Duration;

/// Mini-app card information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiniappCard {
    /// Mini-app AppID.
    pub appid: String,
    /// Mini-app name.
    pub appname: String,
    /// Icon URL.
    pub icon: String,
    /// Card display type or view identifier; the exact meaning is determined by the upstream.
    pub view: String,
    /// Card description.
    pub desc: String,
    /// Mini-app QR code URL.
    pub qrcode: String,
}

impl BotContext {
    /// Requests the mini-app card contained in the specified message and waits up to the given
    /// time for the response data.
    ///
    /// # Arguments
    ///
    /// - `message_id`: ID of the message containing the mini-app card.
    /// - `timeout`: response wait duration after the request has been sent.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when the current connection is unavailable
    /// or `retcode` is non-zero; returns [`crate::error::Error::WebSocketError`] when the
    /// WebSocket send fails; returns [`crate::error::Error::TimeoutError`] on timeout; returns
    /// [`crate::error::Error::JsonError`] when the response cannot be deserialized into
    /// [`MiniappCard`].
    pub async fn get_miniapp_card_with_timeout(
        &self,
        message_id: i64,
        timeout: Duration,
    ) -> Result<MiniappCard> {
        let response = self
            .websocket_send(
                "get_miniapp_card",
                json!({
                    "message_id": message_id,
                }),
            )
            .await?;
        let data = response.data(timeout).await?;
        let card: MiniappCard = serde_json::from_value(data)?;
        Ok(card)
    }

    /// Calls [`Self::get_miniapp_card_with_timeout`] with a ten-second response wait.
    ///
    /// See the timeout variant for other behavior and errors.
    pub async fn get_miniapp_card(&self, message_id: i64) -> Result<MiniappCard> {
        self.get_miniapp_card_with_timeout(message_id, Duration::from_secs(10))
            .await
    }
}

