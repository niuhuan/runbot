//! Send a group AI voice.
//!
//! Sends a `send_group_ai_voice` request and returns a response handle without waiting for
//! the business result.

use crate::error::Result;
use crate::prelude::BotContext;
use crate::prelude::EchoAsyncResponse;
use serde_json::json;

impl BotContext {
    /// Send a group AI voice request and return a handle for waiting on the response.
    ///
    /// An `Ok` return only means the request has been written to the current connection; it
    /// does not mean the voice has been generated or sent.
    ///
    /// # Arguments
    ///
    /// - `group_id`: Target group number.
    /// - `text`: Text to convert.
    /// - `person`: Voice person ID; when `None`, the `person` field is not sent and the
    ///   upstream uses the default voice (the exact behavior depends on the implementation).
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when the current connection is
    /// unavailable; returns [`crate::error::Error::WebSocketError`] when the WebSocket send
    /// fails.
    pub async fn send_group_ai_voice(
        &self,
        group_id: i64,
        text: &str,
        person: Option<i32>,
    ) -> Result<EchoAsyncResponse> {
        let mut params = json!({
            "group_id": group_id,
            "text": text,
        });
        if let Some(p) = person {
            params["person"] = json!(p);
        }
        self.websocket_send("send_group_ai_voice", params).await
    }
}

