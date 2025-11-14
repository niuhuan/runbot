use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use tokio::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiVoice {
    pub file: String,
}

impl BotContext {
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

    pub async fn get_ai_voice(&self, text: &str, person: i32) -> Result<AiVoice> {
        self.get_ai_voice_with_timeout(text, person, Duration::from_secs(30))
            .await
    }
}

