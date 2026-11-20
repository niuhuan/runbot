//! Wrapper for the OneBot `translate_en_to_zh` action. Methods in this module send a
//! request, wait for the response, and deserialize the response data into
//! [`TranslateResult`].

use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use tokio::time::Duration;

/// Response data for `translate_en_to_zh`.
///
/// Returned by [`BotContext::translate_en_to_zh`] and
/// [`BotContext::translate_en_to_zh_with_timeout`] after deserializing the response `data`
/// field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslateResult {
    /// Translated Chinese text, corresponding to the response `data.text`.
    ///
    /// This field has no Serde default value. If the response `data` is missing the `text`
    /// field or `text` is not a string, deserialization fails and returns
    /// [`crate::error::Error::JsonError`].
    pub text: String,
}

impl BotContext {
    /// Requests translation of English text into Chinese and waits for the response within
    /// the specified time.
    ///
    /// This method sends a `translate_en_to_zh` request, waits for the response matching
    /// the echo, checks `retcode`, and deserializes `data` into [`TranslateResult`].
    /// `timeout` limits only the wait for the response after the request has been sent; it
    /// does not include serialization, lock acquisition, or the WebSocket write. This
    /// action is an upstream implementation extension, and upstream support varies.
    ///
    /// # Arguments
    ///
    /// - `text`: English text to translate, sent as-is. This library does not check the
    ///   language, length, or content.
    /// - `timeout`: how long to wait for the response after the request has been sent.
    ///
    /// # Errors
    ///
    /// - Returns [`crate::error::Error::StateError`] when no usable connection is available,
    ///   when the response channel closes before a response arrives, or when the upstream
    ///   `retcode` is non-zero.
    /// - Returns [`crate::error::Error::WebSocketError`] when the WebSocket write fails.
    /// - Returns [`crate::error::Error::TimeoutError`] when no response arrives within
    ///   `timeout`.
    /// - Returns [`crate::error::Error::JsonError`] when the response `data` cannot be
    ///   deserialized into [`TranslateResult`].
    ///
    /// # Cancellation
    ///
    /// Dropping the waiting future ends the local wait and releases the response
    /// registration; a request already sent to the upstream is not withdrawn, and this
    /// cannot be used to assert that the translation did not run.
    pub async fn translate_en_to_zh_with_timeout(
        &self,
        text: &str,
        timeout: Duration,
    ) -> Result<TranslateResult> {
        let response = self
            .websocket_send(
                "translate_en_to_zh",
                json!({
                    "text": text,
                }),
            )
            .await?;
        let data = response.data(timeout).await?;
        let result: TranslateResult = serde_json::from_value(data)?;
        Ok(result)
    }

    /// Requests translation of English text into Chinese and waits up to ten seconds for
    /// the response.
    ///
    /// This method is equivalent to calling [`Self::translate_en_to_zh_with_timeout`] with
    /// a 10-second timeout. See that method for arguments, errors, and cancellation
    /// behavior.
    pub async fn translate_en_to_zh(&self, text: &str) -> Result<TranslateResult> {
        self.translate_en_to_zh_with_timeout(text, Duration::from_secs(10))
            .await
    }
}

