//! Get an AI voice file.
//!
//! The default version uses a thirty-second response wait; voice generation is performed by
//! the upstream.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use tokio::time::Duration;

/// AI voice file information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiVoice {
    /// Voice file identifier, which may be a path, URL, or an upstream-internal ID.
    pub file: String,
}

impl BotContext {
    /// Request AI voice generation and wait for the response data within the specified
    /// timeout.
    ///
    /// # Arguments
    ///
    /// - `text`: Text to convert.
    /// - `person`: Voice person ID; available values can be obtained via
    ///   [`crate::onebot11_api::get_ai_voice_person`], and the exact range is defined by the
    ///   upstream.
    /// - `timeout`: Response wait duration after the request has been sent.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when the current connection is
    /// unavailable or `retcode` is non-zero; returns
    /// [`crate::error::Error::WebSocketError`] when the WebSocket send fails; returns
    /// [`crate::error::Error::TimeoutError`] on timeout; returns
    /// [`crate::error::Error::JsonError`] when the response cannot be deserialized into
    /// [`AiVoice`].
    pub async fn get_ai_voice_with_timeout(
        &self,
        text: &str,
        person: i32,
        timeout: Duration,
    ) -> Result<AiVoice> {
        let response = self
            .websocket_send(
                "get_ai_voice",
                json!({
                    "text": text,
                    "person": person,
                }),
            )
            .await?;
        let data = response.data(timeout).await?;
        let voice: AiVoice = serde_json::from_value(data)?;
        Ok(voice)
    }

    /// Call [`Self::get_ai_voice_with_timeout`] with a thirty-second response wait.
    ///
    /// See the timeout version for other behavior and errors.
    pub async fn get_ai_voice(&self, text: &str, person: i32) -> Result<AiVoice> {
        self.get_ai_voice_with_timeout(text, person, Duration::from_secs(30))
            .await
    }
}

