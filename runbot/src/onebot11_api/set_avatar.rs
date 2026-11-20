//! Wrapper for the OneBot `set_avatar` action. Methods in this module only send requests
//! and do not wait for an upstream response.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_json::json;

impl BotContext {
    /// Requests that the current account's avatar be set to the specified file.
    ///
    /// This method only sends a `set_avatar` request and does not wait for an upstream
    /// response. `Ok(())` only means the request was written to the current WebSocket
    /// connection; it does not mean the avatar has changed. This action is an upstream
    /// implementation extension, and upstream support varies.
    ///
    /// # Arguments
    ///
    /// - `file`: avatar file identifier, sent as-is. Acceptable formats (local path, URL,
    ///   and so on) are determined by the upstream implementation. This library does not
    ///   read the file contents or validate the format or size.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::StateError`] when no usable connection is available,
    /// and [`crate::error::Error::WebSocketError`] when the WebSocket write fails. Invalid
    /// file formats and upstream rejections are not returned by this method.
    pub async fn set_avatar(&self, file: &str) -> Result<()> {
        self.websocket_send(
            "set_avatar",
            json!({
                "file": file,
            }),
        )
        .await?;
        Ok(())
    }
}

