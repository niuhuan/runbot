//! Sends a poke.
//!
//! All three methods only send the request and do not wait for a OneBot business response;
//! `send_poke` and `group_poke` send the same parameters under different action names, and
//! compatibility depends on the upstream implementation.

use crate::error::Result;
use crate::prelude::BotContext;

impl BotContext {
    /// Sends a poke request to a friend.
    ///
    /// Returning `Ok(())` only means the request was written to the current connection; it
    /// does not mean the recipient received it or that the UI displayed it.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when no connection is available, or
    /// [`crate::error::Error::WebSocketError`] when the WebSocket send fails.
    pub async fn friend_poke(&self, user_id: i64) -> Result<()> {
        self.websocket_send(
            "friend_poke",
            serde_json::json!({
                "user_id": user_id,
            }),
        )
        .await?;
        Ok(())
    }
    /// Sends a poke request to the specified member in a group (action `group_poke`).
    ///
    /// Returning `Ok(())` only means the request was written to the current connection; it
    /// does not mean the poke took effect.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when no connection is available, or
    /// [`crate::error::Error::WebSocketError`] when the WebSocket send fails.
    pub async fn group_poke(&self, group_id: i64, user_id: i64) -> Result<()> {
        self.websocket_send(
            "group_poke",
            serde_json::json!({
                "group_id": group_id,
                "user_id": user_id,
            }),
        )
        .await?;
        Ok(())
    }
    /// Sends a poke request to the specified member in a group (action `send_poke`).
    ///
    /// The parameters are the same as [`Self::group_poke`]; only the action name differs.
    /// Whether the upstream supports it depends on the implementation. Returning `Ok(())`
    /// does not mean the poke took effect.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when no connection is available, or
    /// [`crate::error::Error::WebSocketError`] when the WebSocket send fails.
    pub async fn send_poke(&self, group_id: i64, user_id: i64) -> Result<()> {
        self.websocket_send(
            "send_poke",
            serde_json::json!({
                "group_id": group_id,
                "user_id": user_id,
            }),
        )
        .await?;
        Ok(())
    }
}
