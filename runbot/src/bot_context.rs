use std::fmt::{self, Debug};
use std::sync::Arc;
use std::vec;

use crate::error::{Error, Result};
use crate::event::*;
use crate::process::{Processor, loop_processors};
use async_trait::async_trait;
use dashmap::DashMap;
use futures_util::SinkExt;
use futures_util::stream::SplitSink;
use serde_json::json;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::Mutex;
use tokio::time::Duration;
use tokio_tungstenite::WebSocketStream;

// todo: default time for bot context
#[derive(Debug)]
pub struct BotContext {
    pub(crate) connection: Mutex<Option<BotConnection>>,
    pub url: Option<String>,
    pub id: i64,
    pub processors: Arc<Vec<Processor>>,
    pub echo_notifer: Arc<DashMap<String, tokio::sync::mpsc::Sender<Response>>>,
    pub(crate) shutdown_tx: Mutex<Option<tokio::sync::watch::Sender<bool>>>,
    pub(crate) shutdown_rx: Mutex<Option<tokio::sync::watch::Receiver<bool>>>,
}

#[derive(Debug)]
pub struct EchoAsyncResponse(
    String,
    tokio::sync::mpsc::Receiver<Response>,
    Arc<DashMap<String, tokio::sync::mpsc::Sender<Response>>>,
);

impl Drop for EchoAsyncResponse {
    fn drop(&mut self) {
        self.2.remove(&self.0);
    }
}

impl EchoAsyncResponse {
    pub async fn response(mut self, timeout: Duration) -> Result<Response> {
        let r = tokio::time::timeout(timeout, async { self.1.recv().await }).await;
        let r = match r {
            Ok(r) => r,
            Err(err) => {
                return Err(Error::TimeoutError(
                    format!("response timeout for wait echo : {}", self.0),
                    err,
                ));
            }
        };
        match r {
            Some(r) => Ok(r),
            None => Err(Error::StateError(format!(
                "response not received : {}",
                self.0
            ))),
        }
    }

    pub async fn data(self, timeout: Duration) -> Result<serde_json::Value> {
        let r = self.response(timeout).await?;
        if r.retcode != 0 {
            return Err(Error::StateError(r.message));
        } else {
            Ok(r.data)
        }
    }

    pub async fn ok(self, timeout: Duration) -> Result<()> {
        let r = self.response(timeout).await?;
        if r.retcode != 0 {
            return Err(Error::StateError(r.message));
        } else {
            Ok(())
        }
    }
}

impl BotContext {
    pub async fn websocket_send(
        &self,
        action: &str,
        msg: serde_json::Value,
    ) -> Result<EchoAsyncResponse> {
        let echo = uuid::Uuid::new_v4().to_string();
        let (sender, receiver) = tokio::sync::mpsc::channel::<Response>(1);
        self.echo_notifer.insert(echo.clone(), sender);
        let echo_response = EchoAsyncResponse(echo.clone(), receiver, self.echo_notifer.clone());
        let msg = json!(
            {
                "action": action,
                "params": msg,
                "echo": echo
            }
        );
        let msg = serde_json::to_string(&msg).unwrap();
        tracing::debug!("WS send: {}", msg);
        let mut connection_lock = self.connection.lock().await;
        let connection = connection_lock
            .as_mut()
            .ok_or(Error::StateError("connection not ready".to_string()))?;
        connection.send_raw(msg).await?;
        Ok(echo_response)
    }
}

impl BotContext {
    pub(crate) async fn set_connection(&self, connection: impl Into<Option<BotConnection>>) {
        let mut connection_lock = self.connection.lock().await;
        *connection_lock = connection.into();
    }

    pub(crate) async fn handle_receive(
        &self,
        bot_ctx: Arc<BotContext>,
        msg: &tokio_tungstenite::tungstenite::protocol::Message,
    ) {
        match msg {
            tokio_tungstenite::tungstenite::protocol::Message::Text(text) => {
                tracing::debug!("WS received: {}", text.to_string());
                match parse_post(text) {
                    Ok(post) => {
                        tracing::debug!("parse post: {:?}", post);
                        if let Post::Response(response) = &post {
                            if let Some(v) = bot_ctx.echo_notifer.remove(&response.echo) {
                                match v.1.send(response.clone()).await {
                                    Ok(_) => {}
                                    Err(err) => {
                                        tracing::warn!("echo map error : {:?}", err);
                                    }
                                }
                            }
                        }
                        let _ = loop_processors(bot_ctx, self.processors.iter(), &post).await;
                    }
                    Err(e) => {
                        tracing::error!("Parse post error: {:?}", e);
                    }
                }
            }
            _ => {
                tracing::error!("WS received: {:?}", msg);
            }
        }
    }

    pub fn processors(&self) -> Arc<Vec<Processor>> {
        self.processors.clone()
    }

    /// 检查是否已经 shutdown
    pub fn is_shutdown(&self) -> bool {
        // 如果 shutdown_rx 为 None，说明已经 shutdown
        let shutdown_rx_guard = self.shutdown_rx.try_lock();
        if let Ok(shutdown_rx) = shutdown_rx_guard {
            if shutdown_rx.is_none() {
                return true;
            }
            if let Some(receiver) = shutdown_rx.as_ref() {
                return *receiver.borrow();
            }
        }
        false
    }

    /// 关闭 bot，停止所有循环
    pub async fn shutdown(&self) -> Result<()> {
        let mut shutdown_tx = self.shutdown_tx.lock().await;
        if let Some(ref sender) = *shutdown_tx {
            sender.send(true).map_err(|_| {
                Error::StateError("Failed to send shutdown signal".to_string())
            })?;
            // 清空 sender 和 receiver，标记为已 shutdown
            *shutdown_tx = None;
            let mut shutdown_rx = self.shutdown_rx.lock().await;
            *shutdown_rx = None;
        }
        // 断开连接
        self.set_connection(None).await;
        Ok(())
    }
}

#[async_trait]
pub trait WsWriter {
    async fn send_raw(&mut self, msg: String) -> Result<()>;
}

// 泛型实现
#[async_trait]
impl<S> WsWriter
    for SplitSink<WebSocketStream<S>, tokio_tungstenite::tungstenite::protocol::Message>
where
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    async fn send_raw(&mut self, msg: String) -> Result<()> {
        self.send(tokio_tungstenite::tungstenite::protocol::Message::Text(
            msg.into(),
        ))
        .await?;
        Ok(())
    }
}

pub(crate) struct BotConnection {
    pub(crate) sender: Box<dyn WsWriter + Send + Sync>,
}

impl BotConnection {
    pub async fn send_raw(&mut self, msg: String) -> Result<()> {
        self.sender.send_raw(msg).await?;
        Ok(())
    }
}

impl fmt::Debug for BotConnection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BotConnection")
            .field("sender", &"Box<dyn WsWriter>")
            .finish()
    }
}
pub struct BotContextBuilder {
    pub url: Option<String>,
    pub processors: Vec<Processor>,
}

impl BotContextBuilder {
    pub fn new() -> Self {
        Self {
            url: None,
            processors: vec![],
        }
    }

    pub fn url(self, url: impl Into<String>) -> Self {
        Self {
            url: Some(url.into()),
            ..self
        }
    }

    pub fn add_processor(
        mut self,
        processor: impl Into<Processor> + Sync + Send + 'static,
    ) -> Self {
        self.processors.push(processor.into());
        self
    }

    pub fn build(self) -> Result<Arc<BotContext>> {
        let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
        Ok(Arc::new(BotContext {
            connection: Mutex::new(None),
            url: self.url,
            id: 0,
            processors: Arc::new(self.processors),
            echo_notifer: Arc::new(DashMap::new()),
            shutdown_tx: Mutex::new(Some(shutdown_tx)),
            shutdown_rx: Mutex::new(Some(shutdown_rx)),
        }))
    }
}

pub struct BotServer {
    pub bind: String,
    pub processors: Arc<Vec<Processor>>,
}

pub struct BotServerBuilder {
    pub bind: Option<String>,
    pub processors: Vec<Processor>,
}

impl BotServerBuilder {
    pub fn new() -> Self {
        Self {
            bind: None,
            processors: vec![],
        }
    }

    pub fn bind(mut self, bind: impl Into<String>) -> Self {
        self.bind = Some(bind.into());
        self
    }

    pub fn add_processor(
        mut self,
        processor: impl Into<Processor> + Sync + Send + 'static,
    ) -> Self {
        self.processors.push(processor.into());
        self
    }

    pub fn build(self) -> Result<Arc<BotServer>> {
        Ok(Arc::new(BotServer {
            bind: if let Some(bind) = self.bind {
                bind
            } else {
                return Err(Error::ParamsError("bind must be set".to_string()));
            },
            processors: Arc::new(self.processors),
        }))
    }
}
