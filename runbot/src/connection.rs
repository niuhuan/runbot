use std::sync::Arc;

use crate::bot_context::*;
use crate::error::{Error, Result};
use dashmap::DashMap;
use futures_util::StreamExt;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio::time::{Duration, sleep};
use tokio_tungstenite::{WebSocketStream, accept_async, connect_async};

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
        let processors = bot_server.processors.clone();
        tokio::spawn(async move {
            // 协议升级为 WebSocket
            let ws_stream = accept_async(stream).await.unwrap();
            loop_bot(
                Arc::new(BotContext {
                    connection: Mutex::new(None),
                    url: None,
                    id: 0,
                    processors,
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
