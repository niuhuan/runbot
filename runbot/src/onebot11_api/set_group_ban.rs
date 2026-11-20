//! Mute a group member.
//!
//! This module only sends the mute request and does not wait for the OneBot business
//! response.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_json::json;

impl BotContext {
    /// Send a request to set the mute duration for the specified group member.
    ///
    /// A usable WebSocket connection should already be established before calling. This
    /// method does not wait for the OneBot business response; returning `Ok(())` only means
    /// the request was sent successfully, not that the mute has taken effect.
    ///
    /// # Arguments
    ///
    /// - `group_id`: Target group number.
    /// - `user_id`: QQ number of the target group member.
    /// - `duration`: Mute duration in seconds as interpreted by the OneBot protocol; `0`
    ///   means lift the mute. This method does not validate locally whether the upstream
    ///   accepts the value.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when the current connection is
    /// unavailable; returns [`crate::error::Error::WebSocketError`] when the WebSocket send
    /// fails. Failures from upstream permission or parameter validation are not waited for
    /// or returned by this method.
    ///
    /// # Examples
    ///
    /// The following function receives a context whose connection the caller has already
    /// established and sends a one-minute mute request.
    ///
    /// ```no_run
    /// use runbot::bot_context::BotContext;
    ///
    /// async fn request_mute(bot: &BotContext) -> runbot::error::Result<()> {
    ///     bot.set_group_ban(123456, 654321, 60).await?;
    ///     Ok(())
    /// }
    /// # fn main() {}
    /// ```
    pub async fn set_group_ban(&self, group_id: i64, user_id: i64, duration: i64) -> Result<()> {
        let msg = json!(
            {
                "group_id": group_id,
                "user_id": user_id,
                "duration": duration,
            }
        );
        self.websocket_send("set_group_ban", msg).await?;
        Ok(())
    }
}
