use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("params error: {0}")]
    ParamsError(String),
    #[error("parse message error")]
    JsonError(#[from] serde_json::Error),
    #[error("field error: {0}")]
    FieldError(String),
    #[error("state error: {0}")]
    StateError(String),
    #[error("websocket error: {0}")]
    WebSocketError(#[from] tokio_tungstenite::tungstenite::Error),
    #[error("timeout error: {0}")]
    TimeoutError(#[from] tokio::time::error::Elapsed),
}

pub type Result<T> = std::result::Result<T, Error>;
