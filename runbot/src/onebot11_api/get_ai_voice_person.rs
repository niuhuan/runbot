use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use tokio::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiVoicePerson {
    pub person_id: i32,
    pub person_name: String,
}

impl BotContext {
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

    pub async fn get_ai_voice_person(&self) -> Result<Vec<AiVoicePerson>> {
        self.get_ai_voice_person_with_timeout(Duration::from_secs(10))
            .await
    }
}

