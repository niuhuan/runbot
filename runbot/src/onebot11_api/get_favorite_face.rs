//! Gets the favorite face list (upstream extension action `get_favorite_face`).
//!
//! The default version waits up to ten seconds for a response; the `_with_timeout` version
//! allows a custom response wait duration.

use crate::error::Result;
use crate::prelude::BotContext;
use serde_derive::{Deserialize, Serialize};
use tokio::time::Duration;

/// An entry in the favorite face list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FavoriteFace {
    /// Favorite face ID, used to reference the face in a message; the encoding is determined
    /// by the upstream implementation.
    pub face_id: i32,
    /// Name or description of the favorite face, provided by the upstream.
    pub face_name: String,
}

impl BotContext {
    /// Requests the favorite face list and waits for the response data within the specified
    /// time.
    ///
    /// An available connection should exist before calling this method. `timeout` limits the
    /// response wait after the request has been sent; it does not include serialization,
    /// lock acquisition, or the WebSocket write. This method does not retry and does not sort
    /// the returned list.
    ///
    /// # Errors
    ///
    /// - Returns [`crate::error::Error::StateError`] when no connection is available or the
    ///   response channel is closed.
    /// - Returns [`crate::error::Error::WebSocketError`] when the WebSocket send fails.
    /// - Returns [`crate::error::Error::TimeoutError`] when no response arrives within
    ///   `timeout`.
    /// - Returns [`crate::error::Error::StateError`] when the upstream response has a
    ///   non-zero `retcode`.
    /// - Returns [`crate::error::Error::JsonError`] when the response data cannot be
    ///   deserialized into a list of [`FavoriteFace`].
    ///
    /// # Cancellation
    ///
    /// Dropping the pending future stops the local wait and releases the response
    /// registration; the request already sent is not revoked from the upstream.
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

    /// Calls [`Self::get_favorite_face_with_timeout`] with a ten-second response wait.
    ///
    /// Connection, send, timeout, `retcode`, and deserialization errors are the same as in
    /// the version with a timeout; the list order is determined by the upstream response.
    pub async fn get_favorite_face(&self) -> Result<Vec<FavoriteFace>> {
        self.get_favorite_face_with_timeout(Duration::from_secs(10))
            .await
    }
}

