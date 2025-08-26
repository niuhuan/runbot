use anyhow::Result;
use runbot::prelude::*;
use std::{sync::Arc, vec};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();
    let bot_ctx = BotContextBuilder::new()
        .url("ws://localhost:3001")
        .add_message_processor(DEMO_PROCESSOR_FN)
        .build()
        .unwrap();
    loop_bot(bot_ctx).await;
}

#[message_processor]
pub async fn demo_processor_fn(bot_ctx: Arc<BotContext>, message: Arc<Message>) -> Result<bool> {
    if message.raw_message.eq("hello") {
        if let MessageSubType::Friend = message.sub_type {
            bot_ctx
                .send_private_message(message.user_id, "world".to_string())
                .await?;
            let mut chain = vec![];
            chain.push(MessageText::new("this").into());
            chain.push("is face".into());
            chain.push(
                MessageFace {
                    id: "187".to_string(),
                    sub_type: 1,
                }
                .into(),
            );
            bot_ctx.send_private_message(message.user_id, chain).await?;
            let exec_path = std::env::current_dir().unwrap().join("target/test.png");
            bot_ctx.send_private_message(message.user_id, vec![
                MessageImage::new(exec_path.to_str().unwrap()).into(),
            ]).await?;
        }
    }
    Ok(true)
}
