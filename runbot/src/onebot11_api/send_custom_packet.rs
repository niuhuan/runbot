//! Wraps the OneBot action `send_custom_packet`.
//!
//! The method writes a custom command and payload to the current WebSocket
//! connection and returns without waiting for the upstream's business response.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_json::json;

impl BotContext {
    /// Sends a custom protocol packet.
    ///
    /// `command` and `data` are written into the request `params` unchanged;
    /// this library does not validate the command name, the payload structure, or
    /// whether the target implementation supports it. `Ok(())` means only that
    /// the request has been written to the current WebSocket; it does not mean
    /// the upstream has processed or executed it.
    ///
    /// # Arguments
    ///
    /// - `command`: custom command name defined by the target implementation.
    /// - `data`: JSON payload sent with the command; its content is defined by
    ///   the command.
    ///
    /// # Notes
    ///
    /// This is an action provided by an upstream extension and is not part of the
    /// standard OneBot 11 public API. An incorrect command or payload may affect
    /// the account or protocol state; callers should confirm the commands and
    /// formats supported by the target upstream.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when the current connection is
    /// unavailable (not connected, disconnected, or closed);
    /// [`crate::error::Error::WebSocketError`] when the WebSocket write fails.
    /// Whether the upstream recognizes the command is not awaited or returned by
    /// this method.
    pub async fn send_custom_packet(&self, command: &str, data: serde_json::Value) -> Result<()> {
        self.websocket_send(
            "send_custom_packet",
            json!({
                "command": command,
                "data": data,
            }),
        )
        .await?;
        Ok(())
    }
}

