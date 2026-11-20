//! Unified error type.
//!
//! Connection, event parsing, and OneBot API wrappers all use [`enum@Error`] and [`Result`]
//! from this module. Errors can originate from local configuration validation, event or
//! response deserialization, WebSocket transport, or waiting for a remote response.

use thiserror::Error;

/// Errors returned by runbot operations.
#[derive(Error, Debug)]
pub enum Error {
    /// Local parameters are invalid, such as a missing bind address, a missing URL, or
    /// an empty access token.
    #[error("params error: {0}")]
    ParamsError(String),
    /// JSON text or a data structure could not be deserialized into the target type.
    #[error("parse message error")]
    JsonError(#[from] serde_json::Error),
    /// JSON data is missing a required field or a field has an unexpected type.
    #[error("field error: {0}")]
    FieldError(String),
    /// The current object state does not allow the operation, such as a connection that
    /// is not ready, a closed connection, a closed response channel, or a non-zero
    /// `retcode` returned by the upstream.
    #[error("state error: {0}")]
    StateError(String),
    /// A WebSocket handshake, read, or write failed.
    #[error("websocket error: {0}")]
    WebSocketError(#[from] tokio_tungstenite::tungstenite::Error),
    /// No matching API response arrived within the given time.
    ///
    /// The first string is the echo identifier being awaited; the second value is the
    /// Tokio timeout error. This error only means the local wait expired. It does not
    /// prove that the remote side did not execute the operation.
    #[error("timeout error: {0}")]
    TimeoutError(String, tokio::time::error::Elapsed),
}

/// Unified `Result` alias used by runbot operations.
pub type Result<T> = std::result::Result<T, Error>;
