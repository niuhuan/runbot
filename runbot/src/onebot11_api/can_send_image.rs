//! Checks whether the current account can send images (OneBot 11 action `can_send_image`).
//!
//! The method sends the request and waits up to ten seconds for a response.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use tokio::time::Duration;

/// Response data for `can_send_image`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanSendImage {
    /// Whether the current account can send images; the criteria are determined by the
    /// upstream implementation.
    pub yes: bool,
}

impl BotContext {
    /// Requests a check of whether the current account can send images and waits up to ten
    /// seconds for the response data.
    ///
    /// A `false` value for [`CanSendImage::yes`] means the upstream considers image sending
    /// unavailable; it does not indicate a failed request. The criteria (for example,
    /// account permissions or risk-control state) are determined by the upstream
    /// implementation.
    ///
    /// # Errors
    ///
    /// - Returns [`crate::error::Error::StateError`] when no connection is available or the
    ///   response channel is closed.
    /// - Returns [`crate::error::Error::WebSocketError`] when the WebSocket send fails.
    /// - Returns [`crate::error::Error::TimeoutError`] when no response arrives within ten
    ///   seconds.
    /// - Returns [`crate::error::Error::StateError`] when the upstream response has a
    ///   non-zero `retcode`.
    /// - Returns [`crate::error::Error::JsonError`] when the response data cannot be
    ///   deserialized into [`CanSendImage`].
    ///
    /// # Cancellation
    ///
    /// Dropping the pending future stops the local wait and releases the response
    /// registration; the request already sent is not revoked from the upstream.
    pub async fn can_send_image(&self) -> Result<CanSendImage> {
        let response = self
            .websocket_send("can_send_image", serde_json::Value::Null)
            .await?;
        let response = response.data(Duration::from_secs(10)).await?;
        let can_send_image: CanSendImage = serde_json::from_value(response)?;
        Ok(can_send_image)
    }
}
