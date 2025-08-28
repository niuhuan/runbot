use serde_derive::{Serialize, Deserialize};
use crate::prelude::BotContext;
use crate::error::Result;
use tokio::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Image {
    pub file: String,
}

impl BotContext {
    pub async fn get_image(&self, file: &str) -> Result<Image> {
        let response = self.websocket_send("get_image", serde_json::json!({
            "file": file,
        })).await?; 
        let response = response.data(Duration::from_secs(10)).await?;
        let image: Image = serde_json::from_value(response)?;
        Ok(image)
    }
}
