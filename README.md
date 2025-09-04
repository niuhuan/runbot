RUNBOT
======

Rust one bot v11 协议 （ 正向ws / 反向ws ）实现。

- 开箱即用, 使用过程宏定义功能和模块, 轻松实现机器人。
- 具有极高的自由度, 支持模块多层嵌套。

## 开始

以下代码来自 [runbot/examples/client.rs](runbot/examples/client.rs) , 您可以直接跟踪查看。

- 您可以clone项目并运行 `cargo run --example client`  运行正向WS事例
- 您可以clone项目并运行 `cargo run --example server`  运行反向WS事例

#### 0. 引入依赖

```toml
# 使用crates.io版本
runbot = "0"
# 使用github版本
runbot = { git = "https://github.com/niuhuan/runbot.git" }
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

`add_processor`的功能或模块会依次调用, 直至返回true或者返回Error, 会停止链路.

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

## 指南

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

#### 机器人命令


```rust
#[processor(command = "[-|/|~]ban {time:n}[unit:s|m|h]? {user:n}+")]
pub async fn demo_command_ban(
    bot_ctx: Arc<BotContext>,
    message: &Message,
    time: i64,
    unit: Option<String>,
    user: Vec<i64>,
) -> Result<bool> {
    let unit = match unit {
        Some(unit) => match unit.as_str() {
            "s" => 1,
            "m" => 60,
            "h" => 3600,
            _ => unreachable!(),
        },
        None => 1,
    };
    let time = time * unit;
    let msg = format!("禁用用户 {:?} {time}秒", user);
    match message.message_type {
        MessageType::Group => {
            bot_ctx.send_group_message(message.group_id, msg).await?;
        }
        MessageType::Private => {
            bot_ctx.send_private_message(message.user_id, msg).await?;
        }
        _ => {}
    }
    Ok(true)
}
```
- 使用 `#[processor(command = "...")]` 定义命令
- 必须以 bot_ctx: Arc<BotContext>, message: &Message 开头
- 中括号匹配结尾需要为冒号和英文字符
- {:n} 会截从文本开始截取文字 规则为 \d+(\.\d+)? , 截取下的文本以及剩余文本会被trim_space
- {:s} 开始截取文字 规则为 \W+ , 截取下的文本以及剩余文本会被trim_space
- {:e} impl take [\W\w]+  , 截取下的文本以及剩余文本会被trim_space
- 如果需要赋值给变量, 那么在冒号前加入变量名 {name:s} , 最后应用 let some = From::str(截取到的内容), 若成功转换则 赋值name给
- {}? {}+ {}* : 括号结束的后一位特殊符号分别代表: 可选,至少重复1次,重复0或者多次
- ?结尾要使用Option类型当参数, +和*需要使用Vec当作参数 同样会应用FromStr
- [] 表示文字枚举, 使用 | 分割, 同样在前面加入变量名 [name:] 可以赋值给变量, 支持 + 和 * 和 ?
- Tips:
  - 如果@不是全体成员可以映射成数字类型
  - {:s}+ 会一直匹配到结束, 因为数字型属于字符串

## 模块

- 声明模块无需定义struct直接定义一个impl。
- name / help / processors 对应着模块的 名称 / 帮助文本 / 模块功能。
  - 以上参数可以从module宏中省略, name 和 help 会默认为Struct的名称, processors会默认空数组。
  - 可以在impl模块中函数实现trait中的方法, 对函数进行覆盖。
  - 模块也是一个功能(processor), 可以嵌套。
  - ExampleMod代表直接使用字符串, help()代表调用help方法获取, 多个功能用`+`连接
- 机器人不包含菜单功能, 您可以直接使用 BotContext.processors() 获得所有功能, 自由实现您的菜单, 无论是打印还是绘制图片。

```rust
// .add_processor(EXAMPLE_MOD)

#[module(
    name = "ExampleMod",
    help = "help()",
    processors = "mod_process_a+mod_process_b_instance()"
)]
impl Module for ExampleMod {}

// processors = "mod_process_a"
fn help() -> &'static str {
    "我是帮助"
}

#[processor]
async fn mod_process_a(_bot_ctx: Arc<BotContext>, _messgae: &Message) -> Result<bool> {
    Ok(false)
}

#[processor]
async fn mod_process_b(_bot_ctx: Arc<BotContext>, _messgae: &Message) -> Result<bool> {
    Ok(false)
}

fn mod_process_b_instance() -> Processor {
    MOD_PROCESS_B.into()
}
```

## 特性

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
- [x] 拓展
  - [x] 机器人命令匹配
  - [x] 模块

## 环境

#### 需要一个支持onebot v11协议的机器人, 推荐:

- https://llonebot.com/guide/getting-started (Windows)
- https://napneko.github.io/ (Windows/Linux/MacOS)
- https://github.com/NapNeko/NapCat-Docker (docker)

Tips:
  1. windows推荐llonebot一键启动，安装QQ后直接双击启动运行即可。
  2. linux 推荐 NapCat-Docker, 创建/opt/napcat/，chown到自己的用户，按照项目给的docker命令运行, 并且增加以下volumn挂载, 容器删除再次创建时数据得以保留
  - /opt/napcat/.config/QQ/:/app/.config/QQ/         (存放QQ数据，用于自动登录)
  - /opt/napcat/napcat/config/:/app/napcat/config/   (存放napcat配置文件)
  - /opt/napcat/disk/:/opt/napcat/disk/              (存放图片等数据，用于和其他容器文件交互)
  docker容器启动后运行 docker logs -f 扫码登录，并且使用napcat作为密码登录管理后台，设置自动登录的QQ号，下次启动容器时将会自动登录。然后增加一个ws服务端口（例如3001），用于本框架的正向连接。


## 贡献

**Ru**stO**n**e**Bot** 欢迎您的参与！

欢迎提交PR, 以及提出issue。
