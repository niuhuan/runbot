use anyhow::Result;
use runbot::prelude::*;
use std::{sync::Arc, time::Duration};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();
    let bot_ctx = BotContextBuilder::new()
        .url("ws://cool.fubukigroup:3002")
        .add_processor(DEMO_MESSAGE_PROCESSOR_FN)
        .add_processor(DEMO_NOTICE_PROCESSOR_FN)
        .add_processor(DEMO_AUTO_APPROVE_FN)
        .add_processor(DEMO_COMMAND_BAN)
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

// Tips: 设置为允许任何请求添加我时, 会同意好友请求, 并且直接成为单向好友 不会触发此处理器
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

// 中括号匹配结尾需要为冒号和英文字符
// {:n} 会截从文本开始截取文字 规则为 \d+(\.\d+)? , 截取下的文本以及剩余文本会被trim_space
// {:s} 开始截取文字 规则为 \W+ , 截取下的文本以及剩余文本会被trim_space
// {:e} impl take [\W\w]+  , 截取下的文本以及剩余文本会被trim_space
// 如果需要赋值给变量, 那么在冒号前加入变量名 {name:s} , 最后应用 let some = From::str(截取到的内容), 若成功转换则 赋值name给
// {}? {}+ {}* : 括号结束的后一位特殊符号分别代表: 可选,至少重复1次,重复0或者多次
// ?结尾要使用Option类型当参数, +和*需要使用Vec当作参数 同样会应用FromStr
// [] 表示文字枚举, 使用 | 分割, 同样在前面加入变量名 [name:] 可以赋值给变量, 支持 + 和 * 和 ?
// Tips:
// - 如果@不是全体成员可以映射成数字类型
// - {:s}+ 会一直匹配到结束, 因为数字型属于字符串
#[command(pattern = "[-|/|~]ban {time:n}[unit:s|m|h]? {user:n}+")]
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
}
