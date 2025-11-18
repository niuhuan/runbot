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
use futures_util::future::Either;

async fn loop_bot<S>(bot_ctx: Arc<BotContext>, ws_stream: WebSocketStream<S>)
where
    S: AsyncRead + AsyncWrite + Sync + Send + Unpin + 'static,
{
    // 检查是否已经 shutdown
    if bot_ctx.is_shutdown() {
        tracing::warn!("Bot is already shutdown, skipping loop_bot");
        return;
    }

    // 获取 shutdown receiver
    let mut shutdown_rx = {
        let rx_lock = bot_ctx.shutdown_rx.lock().await;
        rx_lock.as_ref().cloned()
    };

    let (ws_sink, mut split_stream) = ws_stream.split();
    let connection = BotConnection {
        sender: Box::new(ws_sink),
    };
    bot_ctx.set_connection(connection).await;
    
    loop {
        // 使用 select! 来同时等待消息和 shutdown 信号
        let shutdown_fut = async {
            if let Some(ref mut rx) = shutdown_rx {
                rx.changed().await.ok();
                *rx.borrow()
            } else {
                false
            }
        };

        let msg_fut = split_stream.next();

        match futures_util::future::select(
            Box::pin(shutdown_fut),
            Box::pin(msg_fut),
        ).await {
            Either::Left((shutdown, _)) => {
                if shutdown {
                    tracing::info!("Shutdown signal received, exiting loop_bot");
                    break;
                }
            }
            Either::Right((msg_option, _)) => {
                match msg_option {
                    Some(Ok(m)) => {
                        let bot_ctx = bot_ctx.clone();
                        _ = tokio::spawn(async move { bot_ctx.handle_receive(bot_ctx.clone(), &m).await });
                    }
                    Some(Err(e)) => {
                        tracing::error!("WS error: {:?}", e);
                        break; // 断开则退出内层循环，重连
                    }
                    None => {
                        tracing::info!("WebSocket stream ended");
                        break;
                    }
                }
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
            let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
            loop_bot(
                Arc::new(BotContext {
                    connection: Mutex::new(None),
                    url: None,
                    id: 0,
                    processors,
                    echo_notifer: Arc::new(DashMap::new()),
                    shutdown_tx: Mutex::new(Some(shutdown_tx)),
                    shutdown_rx: Mutex::new(Some(shutdown_rx)),
                }),
                ws_stream,
            )
            .await;
        });
    }
    Ok(())
}

pub async fn loop_client(bot_ctx: Arc<BotContext>) -> Result<()> {
    // 检查是否已经 shutdown
    if bot_ctx.is_shutdown() {
        return Err(Error::StateError(
            "Bot is already shutdown, cannot start loop_client".to_string(),
        ));
    }

    let url = bot_ctx
        .url
        .as_ref()
        .ok_or(Error::ParamsError(
            "url must be set for loop client".to_string(),
        ))
        .map(|e| e.clone())?;

    // 获取 shutdown receiver
    let mut shutdown_rx = {
        let rx_lock = bot_ctx.shutdown_rx.lock().await;
        rx_lock.as_ref().cloned()
    };

    loop {
        // 检查 shutdown 信号
        if bot_ctx.is_shutdown() {
            tracing::info!("Shutdown signal received, exiting loop_client");
            return Ok(());
        }

        // 使用 select! 来同时等待连接和 shutdown 信号
        let shutdown_fut = async {
            if let Some(ref mut rx) = shutdown_rx {
                rx.changed().await.ok();
                *rx.borrow()
            } else {
                false
            }
        };

        let connect_fut = connect_async(&url);

        match futures_util::future::select(
            Box::pin(shutdown_fut),
            Box::pin(connect_fut),
        ).await {
            Either::Left((shutdown, _)) => {
                if shutdown {
                    tracing::info!("Shutdown signal received, exiting loop_client");
                    return Ok(());
                }
            }
            Either::Right((connect_result, _)) => {
                match connect_result {
                    Ok((ws_stream, _)) => {
                        tracing::info!("WS {} Connected!", &url);
                        let _ = loop_bot(bot_ctx.clone(), ws_stream).await;
                    }
                    Err(e) => tracing::error!("WS {} connect error: {:?}", &url, e),
                }
            }
        }

        // 检查 shutdown 信号，如果已 shutdown 则不重连
        if bot_ctx.is_shutdown() {
            tracing::info!("Shutdown signal received, exiting loop_client");
            return Ok(());
        }

        // 使用 select! 来同时等待 sleep 和 shutdown 信号
        let shutdown_fut = async {
            if let Some(ref mut rx) = shutdown_rx {
                rx.changed().await.ok();
                *rx.borrow()
            } else {
                false
            }
        };

        let sleep_fut = sleep(Duration::from_secs(15));

        match futures_util::future::select(
            Box::pin(shutdown_fut),
            Box::pin(sleep_fut),
        ).await {
            Either::Left((shutdown, _)) => {
                if shutdown {
                    tracing::info!("Shutdown signal received during reconnect wait, exiting loop_client");
                    return Ok(());
                }
            }
            Either::Right(_) => {
                tracing::info!("WS {} reconnecting after 15s...", &url);
            }
        }
    }
}
