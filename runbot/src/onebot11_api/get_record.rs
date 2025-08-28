use serde_derive::{Serialize, Deserialize};
use crate::prelude::BotContext;
use crate::error::Result;
use tokio::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    pub file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecordOutFormat {
    // mp3、amr、wma、m4a、spx、ogg、wav、flac
    Mp3,
    Amr,
    Wma,
    M4a,
    Spx,
    Ogg,
    Wav,
    Flac,
}

impl RecordOutFormat {
    pub fn to_string(&self) -> String {
        match self {
            RecordOutFormat::Mp3 => "mp3".to_string(),
            RecordOutFormat::Amr => "amr".to_string(),
            RecordOutFormat::Wma => "wma".to_string(),
            RecordOutFormat::M4a => "m4a".to_string(),
            RecordOutFormat::Spx => "spx".to_string(),
            RecordOutFormat::Ogg => "ogg".to_string(),
            RecordOutFormat::Wav => "wav".to_string(),
            RecordOutFormat::Flac => "flac".to_string(),
        }
    }
}

impl BotContext {
    pub async fn get_record(&self, file: &str, out_format: RecordOutFormat) -> Result<Record> {
        let response = self.websocket_send("get_record", serde_json::json!({
            "file": file,
            "out_format": out_format.to_string(),
        })).await?;
        let response = response.data(Duration::from_secs(10)).await?;
        let record: Record = serde_json::from_value(response)?;
        Ok(record)
    }
}