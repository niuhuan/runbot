//! Clears the upstream cache (upstream extension action `clear_cache`).
//!
//! This module only sends the request and does not wait for a OneBot response.

use crate::error::Result;
use crate::prelude::BotContext;

impl BotContext {
    /// Sends a request to clear the upstream cache.
    ///
    /// This method does not wait for a OneBot response. Returning `Ok(())` only means the
    /// request was written to the current WebSocket connection; it does not mean the cache
    /// has been cleared. The scope of the cleanup and whether it affects data in use are
    /// determined by the upstream implementation.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when no connection is available, or
    /// [`crate::error::Error::WebSocketError`] when the WebSocket write fails. Upstream
    /// rejection or execution failures are not waited for or returned by this method.
    pub async fn clear_cache(&self) -> Result<()> {
        self.websocket_send("clear_cache", serde_json::Value::Null)
            .await?;
        Ok(())
    }
}

