//! Get available AI voice persons.
//!
//! The default version waits up to ten seconds for the response; the order of the person
//! list is determined by the upstream.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use tokio::time::Duration;

/// AI voice person.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiVoicePerson {
    /// Person ID, used for the `person` parameter of [`crate::onebot11_api::get_ai_voice`].
    pub person_id: i32,
    /// Person display name.
    pub person_name: String,
}

impl BotContext {
    /// Request the available AI voice persons and wait for the response data within the
    /// specified timeout.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when the current connection is
    /// unavailable or `retcode` is non-zero; returns
    /// [`crate::error::Error::WebSocketError`] when the WebSocket send fails; returns
    /// [`crate::error::Error::TimeoutError`] on timeout; returns
    /// [`crate::error::Error::JsonError`] when the response cannot be deserialized into a
    /// list of [`AiVoicePerson`].
    pub async fn get_ai_voice_person_with_timeout(
        &self,
        timeout: Duration,
    ) -> Result<Vec<AiVoicePerson>> {
        let response = self
            .websocket_send("get_ai_voice_person", serde_json::Value::Null)
            .await?;
        let data = response.data(timeout).await?;
        let persons: Vec<AiVoicePerson> = serde_json::from_value(data)?;
        Ok(persons)
    }

    /// Call [`Self::get_ai_voice_person_with_timeout`] with a ten-second response wait.
    ///
    /// See the timeout version for other behavior and errors.
    pub async fn get_ai_voice_person(&self) -> Result<Vec<AiVoicePerson>> {
        self.get_ai_voice_person_with_timeout(Duration::from_secs(10))
            .await
    }
}

