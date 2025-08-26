RUNBOT
======

Rust one bot v11 协议 （正向ws）

- [x] 监听消息
- [x] 发送文本消息

## 使用

您可以clone项目并运行 `cargo run --example demo` 


```toml
runbot = { git = "https://github.com/niuhuan/runbot.git" }
```

```rust
use std::sync::Arc;
use anyhow::Result;
use runbot::prelude::*;

#[tokio::main]
async fn main() {
    // 打印日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();
    // 生成一个bot上下文
    let bot_ctx = BotContextBuilder::new()
        // 声明链接地址
        .url("ws://localhost:3001")
        // 注册消息处理器 (UPPER_SNAKE)
        .add_message_processor(DEMO_PROCESSOR_FN)
        .build()
        .unwrap();
    // loop_bot 或者 spawn loop_bot
    loop_bot(bot_ctx).await;
}

// 声明一个处理器, 当收到消息后被调用
// 参数固定为 Arc<BotContext>，Arc<Message>,
// 返回值为 Result<bool>, 当有一个处理器返回Ok(true)或Err()时将会停止递归
// 
// 此demo为收到好友消息，消息为`hello`时，自动回复`world`
#[message_processor]
pub async fn demo_processor_fn(bot_ctx: Arc<BotContext>, message: Arc<Message>) -> Result<bool> {
    if message.raw_message.eq("hello") {
        if let MessageSubType::Friend = message.sub_type {
            bot_ctx.send_private_message(message.user_id, "world".to_string()).await?;
        }
    }
    Ok(true)
}
```

![hello](images/hello.png)

