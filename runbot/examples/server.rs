use anyhow::Result;
use runbot::prelude::*;
use std::{sync::Arc, time::Duration};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();
    let server = BotServerBuilder::new()
        .bind("0.0.0.0:3131")
        .add_processor(DEMO_MESSAGE_PROCESSOR_FN)
        .add_processor(DEMO_NOTICE_PROCESSOR_FN)
        .build()
        .unwrap();
    loop_server(server).await.unwrap();
}

#[processor]
pub async fn demo_message_processor_fn(
    bot_ctx: Arc<BotContext>,
    message: &Message,
) -> Result<bool> {
    if message.raw_message.eq("hello") {
        if let MessageSubType::Friend = message.sub_type {
            let async_response = bot_ctx
                .send_private_message(message.user_id, "world".to_string())
                .await?;
            let bot_ctx = bot_ctx.clone();
            tokio::spawn(async move {
                let msg_id = async_response.wait_response().await.unwrap().message_id;
                tokio::time::sleep(Duration::from_secs(10)).await;
                bot_ctx.delete_msg(msg_id).await.unwrap();
            });
        }
    }
    Ok(true)
}

#[processor]
pub async fn demo_notice_processor_fn(bot_ctx: Arc<BotContext>, notice: &Notice) -> Result<bool> {
    match notice {
        Notice::FriendRecall(friend_recall) => {
            bot_ctx
                .send_private_message(
                    friend_recall.user_id,
                    format!("{} 撤回了一条消息", friend_recall.user_id),
                )
                .await?;
        }
        _ => {}
    }
    Ok(true)
}
