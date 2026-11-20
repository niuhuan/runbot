//! Fetches a voice file.
//!
//! The request waits up to ten seconds for a response and returns the voice file identifier
//! provided by the upstream.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use tokio::time::Duration;

/// Voice file information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    /// Voice file identifier, which may be a path, URL, or upstream-internal ID.
    pub file: String,
}

/// Voice output format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecordOutFormat {
    // mp3, amr, wma, m4a, spx, ogg, wav, flac
    /// MP3 format.
    Mp3,
    /// AMR format.
    Amr,
    /// WMA format.
    Wma,
    /// M4A format.
    M4a,
    /// SPX format.
    Spx,
    /// OGG format.
    Ogg,
    /// WAV format.
    Wav,
    /// FLAC format.
    Flac,
}

impl RecordOutFormat {
    /// Returns the lowercase format name sent to the upstream; for example,
    /// [`RecordOutFormat::Mp3`] returns `"mp3"`.
    ///
    /// This is an inherent method; the type does not implement [`std::fmt::Display`].
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
    /// Requests conversion and retrieval of a voice file and waits up to ten seconds for the
    /// response data.
    ///
    /// # Arguments
    ///
    /// - `file`: voice file identifier, usually from the `file` field of a message segment.
    /// - `out_format`: desired output format, converted to a lowercase string before sending;
    ///   whether it is supported depends on the upstream.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when the current connection is unavailable
    /// or `retcode` is non-zero; returns [`crate::error::Error::WebSocketError`] when the
    /// WebSocket send fails; returns [`crate::error::Error::TimeoutError`] when no response
    /// arrives within ten seconds; returns [`crate::error::Error::JsonError`] when the response
    /// cannot be deserialized into [`Record`].
    pub async fn get_record(&self, file: &str, out_format: RecordOutFormat) -> Result<Record> {
        let response = self
            .websocket_send(
                "get_record",
                serde_json::json!({
                    "file": file,
                    "out_format": out_format.to_string(),
                }),
            )
            .await?;
        let response = response.data(Duration::from_secs(10)).await?;
        let record: Record = serde_json::from_value(response)?;
        Ok(record)
    }
}
