use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use tokio::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FavoriteFace {
    pub face_id: i32,
    pub face_name: String,
}

impl BotContext {
    pub async fn get_favorite_face_with_timeout(
        &self,
        timeout: Duration,
    ) -> Result<Vec<FavoriteFace>> {
        let response = self
            .websocket_send("get_favorite_face", serde_json::Value::Null)
            .await?;
        let data = response.data(timeout).await?;
        let faces: Vec<FavoriteFace> = serde_json::from_value(data)?;
        Ok(faces)
    }

    pub async fn get_favorite_face(&self) -> Result<Vec<FavoriteFace>> {
        self.get_favorite_face_with_timeout(Duration::from_secs(10))
            .await
    }
}

