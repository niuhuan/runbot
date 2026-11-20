use anyhow::Result;
use runbot::prelude::{send_message::SendMessageAsyncResponse, *};
use std::{sync::Arc, time::Duration};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();
    let mut builder = BotContextBuilder::new().url("ws://localhost:3001");
    // Optional: read the upstream access token from an environment variable. When it is not set,
    // no Authorization header is sent.
    // Do not hard-code real tokens in source code.
    if let Ok(token) = std::env::var("RUNBOT_ACCESS_TOKEN") {
        if !token.is_empty() {
            builder = builder.access_token(token);
        }
    }
    let bot_ctx = builder
        .add_processor(DEMO_MESSAGE_PROCESSOR_FN)
        .add_processor(DEMO_NOTICE_PROCESSOR_FN)
        .add_processor(DEMO_AUTO_APPROVE_FN)
        .add_processor(DEMO_COMMAND_BAN)
        .add_processor(EXAMPLE_MOD)
        .add_processor(DEMO_MESSAGE_PROCESSOR_FN_MORE)
        .build()
        .unwrap();
    loop_client(bot_ctx).await.unwrap();
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
            return Ok(true);
        }
    }
    Ok(false)
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
            return Ok(true);
        }
        _ => {}
    }
    Ok(false)
}

// Tip: when the account is configured to allow any friend request, requests are approved
// automatically and become one-way friends, so this processor is not triggered.
#[processor]
pub async fn demo_auto_approve_fn(bot_ctx: Arc<BotContext>, request: &Request) -> Result<bool> {
    match request {
        Request::Friend(friend_request) => {
            bot_ctx
                .set_friend_add_request(friend_request.flag.as_str(), true, None)
                .await?;
            return Ok(true);
        }
        _ => {}
    }
    Ok(false)
}

// A bracketed pattern ends with a colon followed by an ASCII type letter.
// {:n} matches an integer or decimal number at the start of the current token.
// {:s} takes the remainder of the current token as text.
// {:e} takes all remaining tokens, joined by single spaces.
// To bind a value, put a name before the colon: {name:s}. The captured text is converted
// with FromStr and assigned to that parameter.
// A suffix after the closing brace means: ? optional, + one or more, * zero or more.
// A ? parameter must be Option<T>; + and * parameters must be Vec<T>. Elements are also
// converted with FromStr.
// [...] is a literal enum separated by |. [name:...] binds it to a parameter and supports
// ?, +, and *.
// Tips:
// - An @ mention that is not @all can be mapped to a numeric type.
// - {:s}+ can consume to the end because number-like text is also a string.
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

#[processor]
pub async fn demo_message_processor_fn_more(
    bot_ctx: Arc<BotContext>,
    message: &Message,
) -> Result<bool> {
    // Create a group folder.
    if message.raw_message.eq("create dir") {
        if let MessageType::Group = message.message_type {
            let data = bot_ctx
                .create_group_file_folder(message.group_id, "new folder")
                .await?
                .wait(Duration::from_secs(10))
                .await?;
            tracing::error!("create dir data: {:?}", data);
            return Ok(true);
        }
    }
    // Reply and parse a bilibili link.
    if let Some(MessageData::Json(json)) = message.message.first() {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(json.data.as_str()) {
            if let Some(meta) = json.get("meta") {
                if let Some(detail_1) = meta.get("detail_1") {
                    if let Some(qqdocurl) = detail_1.get("qqdocurl") {
                        if let Some(qqdocurl) = qqdocurl.as_str() {
                            if qqdocurl.contains("/b23.tv/") {
                                message
                                    .reply(
                                        bot_ctx,
                                        format!("你发送了一个b23.tv的链接 : {qqdocurl}"),
                                    )
                                    .await?;
                                return Ok(true);
                            }
                        }
                    }
                }
            }
        }
    }
    // Get the group file list.
    if message.raw_message.eq("list files") {
        if let MessageType::Group = message.message_type {
            let data = bot_ctx.get_group_root_files(message.group_id).await?;
            let mut reply = String::new();
            for folder in data.folders {
                reply.push_str(&format!(
                    "文件夹: {} id: {}\n",
                    folder.folder_name, folder.folder_id
                ));
            }
            for file in data.files {
                reply.push_str(&format!("文件: {} id: {}\n", file.file_name, file.file_id));
            }
            if reply.is_empty() {
                reply.push_str("没有文件");
            }
            message.reply(bot_ctx, reply).await?;
            return Ok(true);
        }
    }
    // Group check-in.
    if message.raw_message.eq("set sign") {
        if let MessageType::Group = message.message_type {
            bot_ctx.set_group_sign(message.group_id).await?;
            return Ok(true);
        }
    }
    // Get a group file URL.
    if message.raw_message.eq("get file url") {
        if let MessageType::Group = message.message_type {
            let files = bot_ctx.get_group_root_files(message.group_id).await?.files;
            if files.is_empty() {
                message.reply(bot_ctx, "没有文件").await?;
                return Ok(true);
            }
            // Take the first file.
            let data = bot_ctx
                .get_group_file_url(message.group_id, files[0].file_id.as_str())
                .await?;
            message
                .reply(bot_ctx, format!("文件链接: {}", data.url))
                .await?;
            return Ok(true);
        }
    }
    Ok(false)
}

#[async_trait::async_trait]
pub trait Reply {
    async fn reply(
        &self,
        bot_ctx: Arc<BotContext>,
        chain: impl SendMessage,
    ) -> Result<SendMessageAsyncResponse>;
}

#[async_trait::async_trait]
impl Reply for Message {
    async fn reply(
        &self,
        bot_ctx: Arc<BotContext>,
        chain: impl SendMessage,
    ) -> Result<SendMessageAsyncResponse> {
        match self.message_type {
            MessageType::Private => Ok(bot_ctx.send_private_message(self.user_id, chain).await?),
            MessageType::Group => {
                let mut data = vec![];
                let mut chain = chain.chain();
                data.push(MessageData::Reply(MessageReply {
                    id: self.message_id,
                }));
                data.push(MessageData::At(MessageAt {
                    qq: self.sender.user_id.to_string(),
                    name: self.sender.nickname.clone(),
                }));
                data.append(&mut chain);
                Ok(bot_ctx.send_group_message(self.group_id, data).await?)
            }
            _ => {
                return Err(anyhow::anyhow!("不支持的消息类型"));
            }
        }
    }
}


#[cfg(test)]
mod tests {

    use super::*;
    #[tokio::test]
    async fn test_demo_command_ban() {
        let bot_ctx = BotContextBuilder::new().build().unwrap();
        let message = Message {
            message_type: MessageType::Group,
            message: vec![MessageData::Text(MessageText {
                text: "-ban 10 1234567890".to_string(),
            })],
            ..Default::default()
        };
        let result = DemoCommandBan
            .process_message(bot_ctx, &message)
            .await
            .unwrap();
        assert!(result);
    }

    #[tokio::test]
    async fn test_shutdown() {
        use runbot::connection::loop_client;
        use runbot::error::Error;

        // Create a bot context.
        let bot_ctx = BotContextBuilder::new()
            .url("ws://ubuntuet:3001") // Use a non-existent address because this only tests shutdown.
            .build()
            .unwrap();

        // Check that the initial state is not shutdown.
        assert!(!bot_ctx.is_shutdown());

        // Call shutdown.
        bot_ctx.shutdown().await.unwrap();

        // Check that the state is shutdown afterward.
        assert!(bot_ctx.is_shutdown());

        // Calling shutdown again should succeed (idempotent operation).
        bot_ctx.shutdown().await.unwrap();
        assert!(bot_ctx.is_shutdown());

        // Calling loop_client again should return an error.
        let result = loop_client(bot_ctx.clone()).await;
        assert!(result.is_err());
        match result {
            Err(Error::StateError(msg)) => {
                assert!(msg.contains("already shutdown"));
            }
            _ => panic!("Expected StateError, got different error"),
        }
    }

    #[tokio::test]
    async fn test_shutdown_stops_loop() {
        use runbot::connection::loop_client;
        use tokio::time::{sleep, timeout, Duration};

        // Create a bot context.
        let bot_ctx = BotContextBuilder::new()
            .url("ws://localhost:3001") // Use a non-existent address.
            .build()
            .unwrap();

        // Start loop_client in the background.
        let bot_ctx_clone = bot_ctx.clone();
        let loop_handle = tokio::spawn(async move {
            loop_client(bot_ctx_clone).await
        });

        // Wait briefly so loop_client starts running.
        sleep(Duration::from_millis(100)).await;

        // Call shutdown.
        bot_ctx.shutdown().await.unwrap();

        // Wait for loop_client to exit; this should be quick because the shutdown signal
        // takes effect immediately.
        // Use a timeout so the test cannot wait forever.
        let result = timeout(Duration::from_secs(2), loop_handle).await;
        assert!(result.is_ok(), "loop_client should exit after shutdown");

        // Check that loop_client exited normally (returned Ok).
        let loop_result = result.unwrap();
        assert!(loop_result.is_ok(), "loop_client should return Ok after shutdown");
    }
}
