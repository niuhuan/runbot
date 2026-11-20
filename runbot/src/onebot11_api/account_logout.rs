//! Account logout (upstream extension action `account_logout`).
//!
//! This module only sends the request and does not wait for a OneBot response.

use crate::error::Result;
use crate::prelude::BotContext;

impl BotContext {
    /// Sends a request to log out the current account.
    ///
    /// This method sends no parameters and does not wait for a OneBot response. Returning
    /// `Ok(())` only means the request was written to the current WebSocket connection; it
    /// does not mean the account has logged out or the connection has closed. Reconnection,
    /// offline notices, and related behavior after logout are determined by the upstream
    /// implementation.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when no connection is available, or
    /// [`crate::error::Error::WebSocketError`] when the WebSocket write fails. Upstream
    /// rejection or execution failures are not waited for or returned by this method.
    pub async fn account_logout(&self) -> Result<()> {
        self.websocket_send("account_logout", serde_json::Value::Null)
            .await?;
        Ok(())
    }
}

