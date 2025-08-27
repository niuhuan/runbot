use std::fmt::{self, Debug};
use std::sync::Arc;
use std::vec;

use crate::error::{Error, Result};
use crate::event::*;
use crate::process::MessageProcessor;
use async_trait::async_trait;
use dashmap::DashMap;
use futures_util::stream::SplitSink;
use futures_util::{SinkExt, StreamExt};
use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio::time::{Duration, sleep};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{WebSocketStream, accept_async, connect_async};

#[derive(Debug)]
pub struct BotContext {
    connection: Mutex<Option<BotConnection>>,
    pub url: Option<String>,
    pub id: i64,
    pub message_processors: Arc<Vec<Box<dyn MessageProcessor>>>,
    pub echo_notifer: Arc<DashMap<String, tokio::sync::mpsc::Sender<Response>>>,
}

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
    pub async fn wait_response(mut self, timeout: Duration) -> Result<Response> {
        let r = tokio::time::timeout(timeout, async { self.1.recv().await }).await?;
        Ok(r.ok_or(Error::StateError("response not received".to_string()))?)
    }
}

pub struct SendMessageAsyncResponse(EchoAsyncResponse);

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub struct SendMessageResponse {
    pub message_id: i64,
}

impl SendMessageAsyncResponse {
    pub async fn wait_response(self, timeout: Duration) -> Result<SendMessageResponse> {
        let r = self.0.wait_response(timeout).await?;
        if r.retcode != 0 {
            return Err(Error::StateError(r.message));
        } else {
            Ok(serde_json::from_value(r.data)?)
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

    pub async fn send_private_message(
        &self,
        user_id: i64,
        message: impl SendMessage,
    ) -> Result<SendMessageAsyncResponse> {
        let msg = json!(
            {
                "user_id": user_id,
                "message": message.json()?,
            }
        );
        self.websocket_send("send_private_msg", msg)
            .await
            .map(|r| SendMessageAsyncResponse(r))
    }

    pub async fn send_group_message(
        &self,
        group_id: i64,
        message: impl SendMessage,
    ) -> Result<SendMessageAsyncResponse> {
        let msg = json!(
            {
                "group_id": group_id,
                "message": message.json()?,
            }
        );
        self.websocket_send("send_group_msg", msg)
            .await
            .map(|r| SendMessageAsyncResponse(r))
    }

    pub async fn send_message(
        &self,
        message_type: MessageType,
        target_id: i64,
        message: impl SendMessage,
    ) -> Result<SendMessageAsyncResponse> {
        match message_type {
            MessageType::Private => self.send_private_message(target_id, message).await,
            MessageType::Group => self.send_group_message(target_id, message).await,
        }
    }

    pub async fn delete_msg(&self, message_id: i64) -> Result<EchoAsyncResponse> {
        let msg = json!(
            {
                "message_id": message_id,
            }
        );
        self.websocket_send("delete_msg", msg).await
    }
}

impl BotContext {
    async fn set_connection(&self, connection: impl Into<Option<BotConnection>>) {
        let mut connection_lock = self.connection.lock().await;
        *connection_lock = connection.into();
    }

    async fn handle_receive(&self, bot_ctx: Arc<BotContext>, msg: &Message) {
        match msg {
            Message::Text(text) => {
                tracing::debug!("WS received: {}", text.to_string());
                match parse_post(text) {
                    Ok(post) => match post {
                        Post::MetaEvent(meta_event) => {
                            tracing::debug!("WS received: {:?}", meta_event);
                        }
                        Post::Response(response) => {
                            tracing::debug!("WS received: {:?}", response);
                            if let Some(v) = bot_ctx.echo_notifer.remove(&response.echo) {
                                match v.1.send(response).await {
                                    Ok(_) => {}
                                    Err(err) => {
                                        tracing::warn!("echo map error : {:?}", err);
                                    }
                                }
                            }
                        }
                        Post::Message(message) => {
                            tracing::debug!("WS received: {:?}", message);
                            let message = Arc::new(message);
                            let message_processors = self.message_processors.clone();
                            tokio::spawn(async move {
                                for processor in message_processors.iter() {
                                    let _ = processor
                                        .process_message(bot_ctx.clone(), message.clone())
                                        .await;
                                }
                            });
                        }
                        Post::Notice(_) => {
                            //
                        }
                    },
                    Err(e) => {
                        tracing::error!("WS received: {:?}", e);
                    }
                }
            }
            _ => {
                tracing::error!("WS received: {:?}", msg);
            }
        }
    }
}

#[async_trait]
pub trait WsWriter {
    async fn send_raw(&mut self, msg: String) -> Result<()>;
}

// 泛型实现
#[async_trait]
impl<S> WsWriter for SplitSink<WebSocketStream<S>, Message>
where
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    async fn send_raw(&mut self, msg: String) -> Result<()> {
        self.send(Message::Text(msg.into())).await?;
        Ok(())
    }
}

struct BotConnection {
    sender: Box<dyn WsWriter + Send + Sync>,
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
    pub message_processors: Vec<Box<dyn MessageProcessor>>,
}

impl BotContextBuilder {
    pub fn new() -> Self {
        Self {
            url: None,
            message_processors: vec![],
        }
    }

    pub fn url(self, url: impl Into<String>) -> Self {
        Self {
            url: Some(url.into()),
            ..self
        }
    }

    pub fn add_message_processor(
        mut self,
        processor: impl MessageProcessor + Copy + 'static,
    ) -> Self {
        self.message_processors.push(Box::new(processor));
        self
    }

    pub fn build(self) -> Result<Arc<BotContext>> {
        Ok(Arc::new(BotContext {
            connection: Mutex::new(None),
            url: self.url,
            id: 0,
            message_processors: Arc::new(self.message_processors),
            echo_notifer: Arc::new(DashMap::new()),
        }))
    }
}

pub struct BotServer {
    pub bind: String,
    pub message_processors: Arc<Vec<Box<dyn MessageProcessor>>>,
}

pub struct BotServerBuilder {
    pub bind: Option<String>,
    pub message_processors: Vec<Box<dyn MessageProcessor>>,
}

impl BotServerBuilder {
    pub fn new() -> Self {
        Self {
            bind: None,
            message_processors: vec![],
        }
    }

    pub fn bind(mut self, bind: impl Into<String>) -> Self {
        self.bind = Some(bind.into());
        self
    }

    pub fn add_message_processor(
        mut self,
        processor: impl MessageProcessor + Copy + 'static,
    ) -> Self {
        self.message_processors.push(Box::new(processor));
        self
    }

    pub fn build(self) -> Result<Arc<BotServer>> {
        Ok(Arc::new(BotServer {
            bind: if let Some(bind) = self.bind {
                bind
            } else {
                return Err(Error::ParamsError("bind must be set".to_string()));
            },
            message_processors: Arc::new(self.message_processors),
        }))
    }
}

async fn loop_bot<S>(bot_ctx: Arc<BotContext>, ws_stream: WebSocketStream<S>)
where
    S: AsyncRead + AsyncWrite + Sync + Send + Unpin + 'static,
{
    let (ws_sink, mut split_stream) = ws_stream.split();
    let connection = BotConnection {
        sender: Box::new(ws_sink),
    };
    bot_ctx.set_connection(connection).await;
    while let Some(msg) = split_stream.next().await {
        match msg {
            Ok(m) => {
                let bot_ctx = bot_ctx.clone();
                _ = tokio::spawn(async move { bot_ctx.handle_receive(bot_ctx.clone(), &m).await });
            }
            Err(e) => {
                tracing::error!("WS error: {:?}", e);
                break; // 断开则退出内层循环，重连
            }
        }
    }
    bot_ctx.set_connection(None).await;
}

pub async fn loop_server(bot_server: Arc<BotServer>) -> Result<()> {
    // 监听本地 9001 端口
    let listener = TcpListener::bind(&bot_server.bind).await.unwrap();
    println!("WebSocket server started on ws://{}", &bot_server.bind);
    while let Ok((stream, _)) = listener.accept().await {
        let message_processors = bot_server.message_processors.clone();
        tokio::spawn(async move {
            // 协议升级为 WebSocket
            let ws_stream = accept_async(stream).await.unwrap();
            loop_bot(
                Arc::new(BotContext {
                    connection: Mutex::new(None),
                    url: None,
                    id: 0,
                    message_processors: message_processors,
                    echo_notifer: Arc::new(DashMap::new()),
                }),
                ws_stream,
            )
            .await;
        });
    }
    Ok(())
}

pub async fn loop_client(bot_ctx: Arc<BotContext>) -> Result<()> {
    let url = bot_ctx
        .url
        .as_ref()
        .ok_or(Error::ParamsError(
            "url must be set for loop client".to_string(),
        ))
        .map(|e| e.clone())?;
    loop {
        match connect_async(&url).await {
            Ok((ws_stream, _)) => {
                tracing::info!("WS {} Connected!", &url);
                let _ = loop_bot(bot_ctx.clone(), ws_stream).await;
            }
            Err(e) => tracing::error!("WS {} connect error: {:?}", &url, e),
        }
        tracing::info!("WS {} reconnecting after 15s...", &url);
        sleep(Duration::from_secs(15)).await;
    }
}
