RUNBOT
======

Rust one bot v11 协议 （ 正向ws / 反向ws ）实现。


## 准备环境

https://llonebot.com/guide/getting-started

## 宏定义、开箱即用

#### 0. 引入依赖

```toml
# 使用github版本
runbot = { git = "https://github.com/niuhuan/runbot.git" }

# 使用crates.io版本
runbot = "0"
```

#### 1. 定义事件处理器

```rust
use std::sync::Arc;
use anyhow::Result;
use runbot::prelude::*;

// 声明一个处理器, 当收到消息后被调用
// 参数固定为 Arc<BotContext>，&对应事件类型,  （事件类型包括 Message 消息、Notice 通知、Request 请求、Post 以上三种的枚举） 
// 返回值为 Result<bool>, 当有一个处理器返回Ok(true)或Err()时将会停止递归
// 
// 此demo为收到好友消息，消息为`hello`时，自动回复`world`
#[processor]
pub async fn demo_processor_fn(bot_ctx: Arc<BotContext>, message: &Message) -> Result<bool> {
    if message.raw_message.eq("hello") {
        if let MessageSubType::Friend = message.sub_type {
            bot_ctx.send_private_message(message.user_id, "world".to_string()).await?;
        }
    }
    Ok(true)
}
```

#### 2. 连接bot

```rust
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
        // 注册事件处理器 (方法名的UPPER_SNAKE)
        .add_processor(DEMO_PROCESSOR_FN)
        .build()
        .unwrap();
    // loop_client 或者 spawn loop_client （方便定时任务等）
    loop_client(bot_ctx).await;
}
```

![hello](images/hello.png)

## Tips

- 您可以clone项目并运行 `cargo run --example client`  运行正向WS事例
- 您可以clone项目并运行 `cargo run --example server`  运行反向WS事例

#### 发送消息以及消息链 (文字以及图片)

```rust
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
```

#### 确保消息发送成功、获取消息ID、撤回消息 

```rust
bot_ctx.send_private_message(12345, "hello").await?;
let async_response = bot_ctx.send_private_message(message.user_id, chain).await?;
let msg_id = async_response.wait_response().await?.message_id;
bot_ctx.delete_msg(msg_id).await?;
```

#### 反向WS

```rust
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();
    let server = BotServerBuilder::new()
        .bind("0.0.0.0:3131")
        .add_message_processor(DEMO_PROCESSOR_FN)
        .build()
        .unwrap();
    loop_server(server).await.unwrap();
}
```

## Features

https://llonebot.apifox.cn/

https://github.com/botuniverse/onebot-11/blob/master/api/public.md

- [x] 事件
  - [x] 监听元事件
  - [x] 监听私聊、组群消息
  - [x] 监听通知（群增减员、拍一拍、手气最佳等）
  - [x] 监听请求（好友请求、进群请求）
- [x] 操作
  - [x] 发送文本、图片、自定义onebot11JSON消息 到 私聊、组群 (并异步取得发送结果以及消息ID)
  - [x] 撤回消息、根据消息ID获取消息、获取合并转发消息
  - [x] 群组踢人、禁言、全体禁言、设置管理员、修改群名片、修改群名称、退出（解散）组群、设置群头衔
  - [ ] 群组匿名用户禁言、群组匿名
  - [x] 获取好友列表、获取组群信息、获取组群成员信息、获取组群成员列表
  - [ ] 发送好友赞
  - [x] 同意好友请求、同意进群请求
  - [ ] 获取群荣誉信息
  - [ ] 获取 Cookies、获取 CSRF Token、获取 QQ 相关接口凭证
  - [x] 获取语音、获取图片、检查是否可以发送图片、检查是否可以发送语音
  - [ ] 获取运行状态、获取版本信息、重启 OneBot 实现、清理缓存
  - [ ] ...
  - [x] 删除好友、戳一戳好友、设置群备注
  - [x] 标记消息已读、图片OCR
  - [ ] ...
- [x] 类型
  - [x] 消息：文本、表情、图片、语音、短视频、@某人、回复、合并转发、合并转发自定义节点
  - [ ] 消息: 猜拳魔法表情、掷骰子魔法表情、戳一戳、窗口抖动（戳一戳）、匿名发消息、链接分享、推荐好友、推荐群、位置、音乐分享、音乐自定义分享、合并转发节点、XML 消息、JSON 消息
- [ ] 拓展
  - [ ] 命令匹配、命令宏
