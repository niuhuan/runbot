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

引用:
- https://napcat.apifox.cn/
- https://llonebot.apifox.cn/
- https://github.com/botuniverse/onebot-11/blob/master/api/public.md

### 人工实现的API

- [x] 事件
  - [x] 监听元事件
  - [x] 监听私聊、组群消息
  - [x] 监听通知（群增减员、拍一拍、手气最佳等）
  - [x] 监听请求（好友请求、进群请求）
- [x] 操作
  - [x] 消息
    - [x] 发送文本、图片、自定义onebot11JSON消息 到 私聊、组群 (并异步取得发送结果以及消息ID)
    - [x] 撤回消息、根据消息ID获取消息、获取合并转发消息
    - [x] 获取语音、获取图片、检查是否可以发送图片、检查是否可以发送语音
    - [x] 转发单条私聊、组群消息
    - [x] 标记消息已读、图片OCR
  - [x] 群操作
    - [x] 群组踢人、禁言、全体禁言、设置管理员、修改群名片、修改群名称、退出（解散）组群、设置群头衔
    - [x] 获取组群信息、获取组群成员信息、获取组群成员列表
    - [x] 群戳一戳、设置群备注、群签到
    - [x] 同意进群请求
    - [ ] 群组匿名用户禁言、群组匿名
    - [ ] 获取群荣誉信息
  - [x] 好友
    - [x] 同意好友请求
    - [x] 获取好友列表
    - [x] 删除好友 (llonebot napcat 接口略有不同, llonebot不支持第2、3个参数将会直接忽略)
    - [x] 戳一戳
    - [ ] 发送好友赞
  - [x] 文件
    - [x] 获取私聊文件链接、上传群文件、删除群文件、列出群文件根目录、重命名群文件夹、删除群文件夹、获取群文件链接
  - [x] 认证
    - [ ] 获取 Cookies、获取 CSRF Token、获取 QQ 相关接口凭证
  - [ ] 系统
    - [ ] 获取运行状态、获取版本信息、重启 OneBot 实现、清理缓存
- [x] 类型
  - [x] 消息：文本、表情、图片、语音、短视频、@某人、回复、合并转发、合并转发自定义节点、XML 消息、JSON 消息
  - [ ] 消息: 猜拳魔法表情、掷骰子魔法表情、戳一戳、窗口抖动（戳一戳）、匿名发消息、链接分享、推荐好友、推荐群、位置、音乐分享、音乐自定义分享、合并转发节点
- [x] 拓展
  - [x] 机器人命令匹配
  - [x] 模块

- [x] 快捷回复组群或个人消息, 请参考 [runbot/examples/client.rs](runbot/examples/client.rs) 中的 `pub trait Reply`

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


## 使用AI学习以上代码并实现的剩余的API

#### 账号相关（28个）
- [x] `set_account_profile` - 设置账号信息
- [x] `get_filtered_friend_requests` - 获取被过滤好友请求
- [x] `handle_filtered_friend_request` - 处理被过滤好友请求
- [x] `get_online_clients` - 获取当前账号在线客户端列表
- [x] `set_online_status` - 设置在线状态
- [x] `send_like` - 点赞
- [x] `set_private_msg_read` - 设置私聊已读
- [x] `set_group_msg_read` - 设置群聊已读
- [x] `get_login_info` - 获取登录号信息
- [x] `set_friend_remark` - 设置好友备注
- [x] `get_unidirectional_friend_list` - 获取单向好友列表
- [x] `get_recommended_friends` - 获取推荐好友/群聊卡片
- [x] `get_recommended_groups` - 获取推荐群聊卡片
- [x] `get_friend_group_list` - 获取好友分组列表
- [x] `set_avatar` - 设置头像
- [x] `create_favorite` - 创建收藏
- [x] `set_signature` - 设置个性签名
- [x] `get_recent_contact` - 最近消息列表
- [x] `get_account_info` - 获取账号信息
- [x] `set_all_msg_read` - 设置所有消息已读
- [x] `get_like_list` - 获取点赞列表
- [x] `get_favorite_face` - 获取收藏表情
- [x] `get_online_model` - 获取在线机型
- [x] `set_online_model` - 设置在线机型
- [x] `get_user_status` - 获取用户状态
- [x] `get_status` - 获取状态
- [x] `get_miniapp_card` - 获取小程序卡片
- [x] `set_custom_online_status` - 设置自定义在线状态

#### 消息相关（8个）
- [x] `get_group_history_msg` - 获取群历史消息
- [x] `get_private_history_msg` - 获取好友历史消息
- [x] `send_forward_msg` - 发送合并转发消息（私聊/群聊）
- [x] `send_group_ai_voice` - 发送群AI语音
- [x] `set_essence_msg` - 贴表情
- [x] `get_essence_msg_list` - 获取贴表情详情
- [x] `get_record_detail` - 获取语音消息详情
- [x] `get_image_detail` - 获取图片消息详情

#### 群聊相关（18个）
- [x] `batch_kick_group_member` - 批量踢出群成员
- [x] `get_group_honor` - 获取群荣誉
- [x] `get_group_at_all_remain` - 获取群 @全体成员 剩余次数
- [x] `set_group_search` - 设置群搜索
- [x] `set_group_add_option` - 设置群添加选项
- [x] `set_group_bot_add_option` - 设置群机器人添加选项
- [x] `get_group_system_msg` - 获取群系统消息
- [x] `set_group_avatar` - 设置群头像
- [x] `delete_essence_msg` - 删除群精华消息
- [x] `send_group_notice` - 发送群公告
- [x] `get_group_notice` - 获取群公告
- [x] `delete_group_notice` - 删除群公告
- [x] `get_group_info_ex` - 获取群信息ex
- [x] `get_group_ban_list` - 获取群禁言列表
- [x] `get_group_filter_system_msg` - 获取群过滤系统消息
- [x] `group_check_in` - 群打卡

#### 文件相关（8个）
- [x] `upload_private_file` - 上传私聊文件
- [x] `move_group_file` - 移动群文件
- [x] `save_file_to_cache` - 转存为永久文件
- [x] `rename_group_file` - 重命名群文件
- [x] `get_file_info` - 获取文件信息
- [x] `get_group_file_system_info` - 获取群文件系统信息
- [x] `download_file_to_cache` - 下载文件到缓存目录

#### 密钥相关（7个）
- [x] `get_cookies` - 获取cookies
- [x] `get_csrf_token` - 获取 CSRF Token
- [x] `get_clientkey` - 获取clientkey
- [x] `get_credentials` - 获取 QQ 相关接口凭证
- [x] `nc_get_rkey` - nc获取rkey
- [x] `get_rkey` - 获取rkey
- [x] `get_rkey_service` - 获取rkey服务

#### 个人操作（6个）
- [x] `translate_en_to_zh` - 英译中
- [x] `set_input_status` - 设置输入状态
- [x] `handle_quick_operation` - 对事件执行快速操作
- [x] `get_ai_voice_person` - 获取AI语音人物
- [x] `click_button` - 点击按钮
- [x] `get_ai_voice` - 获取AI语音

#### 系统操作（5个）
- [x] `get_version_info` - 获取版本信息
- [x] `clear_cache` - 清空缓存
- [x] `get_bot_account_range` - 获取机器人账号范围
- [x] `account_logout` - 账号退出
- [x] `send_custom_packet` - 发送自定义组包
- [x] `get_packet_status` - 获取packet状态


## 贡献

**Ru**stO**n**e**Bot** 欢迎您的参与！

欢迎提交PR, 以及提出issue。
