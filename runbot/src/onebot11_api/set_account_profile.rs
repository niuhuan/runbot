//! Wraps the OneBot action `set_account_profile`.
//!
//! The method writes only the profile fields that should be changed to the
//! current WebSocket connection and returns without waiting for the upstream's
//! business response.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_json::json;

impl BotContext {
    /// Requests a change to the current account's profile.
    ///
    /// All five arguments are optional: when `Some`, the corresponding key is
    /// written to `params` as a string; when `None`, the key is omitted from the
    /// request entirely and JSON `null` is not sent. This method does not
    /// validate email format, length, or content. `Ok(())` means only that the
    /// request has been written to the current WebSocket; it does not mean the
    /// profile has been changed by the upstream.
    ///
    /// # Arguments
    ///
    /// - `nickname`: new nickname.
    /// - `company`: company information.
    /// - `email`: email address.
    /// - `college`: college or school information.
    /// - `personal_note`: personal note.
    ///
    /// # Notes
    ///
    /// Whether omitting a key keeps the current value is defined by the upstream
    /// implementation; this interface has no separate way to clear a field.
    /// `Some("")` is sent as an empty string and differs from `None`.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when the current connection is
    /// unavailable (not connected, disconnected, or closed);
    /// [`crate::error::Error::WebSocketError`] when the WebSocket write fails.
    /// Upstream field validation or permission errors are not awaited or
    /// returned by this method.
    pub async fn set_account_profile(
        &self,
        nickname: Option<&str>,
        company: Option<&str>,
        email: Option<&str>,
        college: Option<&str>,
        personal_note: Option<&str>,
    ) -> Result<()> {
        let mut params = serde_json::Map::new();
        if let Some(n) = nickname {
            params.insert("nickname".to_string(), json!(n));
        }
        if let Some(c) = company {
            params.insert("company".to_string(), json!(c));
        }
        if let Some(e) = email {
            params.insert("email".to_string(), json!(e));
        }
        if let Some(c) = college {
            params.insert("college".to_string(), json!(c));
        }
        if let Some(p) = personal_note {
            params.insert("personal_note".to_string(), json!(p));
        }
        self.websocket_send("set_account_profile", json!(params))
            .await?;
        Ok(())
    }
}

