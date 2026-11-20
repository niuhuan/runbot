//! Wraps the OneBot action `handle_quick_operation`.
//!
//! The method writes the event context and quick-operation object to the current
//! WebSocket connection and returns without waiting for the upstream's business
//! response.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_json::json;

impl BotContext {
    /// Performs a quick operation on an event, such as replying directly or
    /// recalling a message.
    ///
    /// Both arguments are placed into `params` unchanged. This library does not
    /// validate their structure or determine whether the operation applies to
    /// the given event. `Ok(())` means only that the request has been written to
    /// the current WebSocket; it does not mean the operation has been executed.
    ///
    /// # Arguments
    ///
    /// - `context`: event context object, usually the complete JSON of the event
    ///   that triggered the quick operation.
    /// - `operation`: operation object to perform; the available fields are
    ///   defined by the event type and the upstream implementation.
    ///
    /// # Notes
    ///
    /// This is a wrapper for the OneBot 11 `handle_quick_operation` action.
    /// Whether the upstream supports it and which operations it supports are
    /// implementation-defined. Use the specific API for the action when explicit
    /// result feedback is needed.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when the current connection is
    /// unavailable (not connected, disconnected, or closed);
    /// [`crate::error::Error::WebSocketError`] when the WebSocket write fails.
    /// Parameter validation or execution failures from the upstream are not
    /// awaited or returned by this method.
    pub async fn handle_quick_operation(
        &self,
        context: serde_json::Value,
        operation: serde_json::Value,
    ) -> Result<()> {
        self.websocket_send(
            "handle_quick_operation",
            json!({
                "context": context,
                "operation": operation,
            }),
        )
        .await?;
        Ok(())
    }
}

