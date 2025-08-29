use anyhow::Result;
use runbot::prelude::*;
use std::sync::Arc;

fn main() {}

// 测试 {:s} 和 ? 的组合
#[command(pattern = "[-|/|~]remind {time:n}[unit:s|m|h]? {action:s}? {message_text:e}")]
pub async fn demo_command_remind(
    bot_ctx: Arc<BotContext>,
    message: &Message,
    time: i64,
    unit: Option<String>,
    action: Option<String>,
    message_text: String,
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
    let action_text = action.unwrap_or_else(|| "提醒".to_string());
    let msg = format!("{time}秒后{action_text}: {message_text}");
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

// 测试 {:n}* 和 [] 的组合
#[command(pattern = "[-|/|~]calc {numbers:n}* [operation:+|-|*|/] {result:n}")]
pub async fn demo_command_calc(
    bot_ctx: Arc<BotContext>,
    message: &Message,
    numbers: Vec<i64>,
    operation: String,
    result: i64,
) -> Result<bool> {
    if numbers.is_empty() {
        return Ok(false);
    }

    let calculated_result = match operation.as_str() {
        "+" => numbers.iter().sum(),
        "-" => numbers.iter().fold(numbers[0], |acc, &x| acc - x),
        "*" => numbers.iter().product(),
        "/" => numbers.iter().fold(numbers[0], |acc, &x| acc / x),
        _ => return Ok(false),
    };

    let msg = if calculated_result == result {
        format!("计算正确! {:?} {} = {}", numbers, operation, result)
    } else {
        format!(
            "计算错误! {:?} {} = {}, 你的答案是 {}",
            numbers, operation, calculated_result, result
        )
    };

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

// 测试 {:s}+ 和 [] 的组合
#[command(pattern = "[-|/|~]tag {tags:s}+ [tag_type:user|group|all]? {message_text:e}")]
pub async fn demo_command_tag(
    bot_ctx: Arc<BotContext>,
    message: &Message,
    tags: Vec<String>,
    tag_type: Option<String>,
    message_text: String,
) -> Result<bool> {
    let tag_type = tag_type.unwrap_or_else(|| "all".to_string());
    let tags_str = tags.join(", ");
    let msg = format!(
        "标签类型: {}, 标签: [{}], 内容: {}",
        tag_type, tags_str, message_text
    );

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

// 测试 {:n}? 和 {:e} 的组合
#[command(pattern = "[-|/|~]echo {count:n}? {text:e}")]
pub async fn demo_command_echo(
    bot_ctx: Arc<BotContext>,
    message: &Message,
    count: Option<i64>,
    text: String,
) -> Result<bool> {
    let count = count.unwrap_or(1);
    let repeated_text = text.repeat(count as usize);
    let msg = format!("重复{}次: {}", count, repeated_text);

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

// 测试复杂的组合：{:n}* {:s}? []+ {:e}
#[command(
    pattern = "[-|/|~]schedule {times:n}* {priority:s}? [actions:work|play|study|rest]+ {description:e}"
)]
pub async fn demo_command_schedule(
    bot_ctx: Arc<BotContext>,
    message: &Message,
    times: Vec<i64>,
    priority: Option<String>,
    actions: Vec<String>,
    description: String,
) -> Result<bool> {
    let times_str = times
        .iter()
        .map(|t| t.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    let priority_text = priority.unwrap_or_else(|| "普通".to_string());
    let actions_str = actions.join(", ");
    let msg = format!(
        "时间: [{}], 优先级: {}, 动作: [{}], 描述: {}",
        times_str, priority_text, actions_str, description
    );

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
mod test {

    use super::*;

    #[tokio::test]
    async fn test_demo_command_remind() {
        let bot_ctx = BotContextBuilder::new().build().unwrap();
        let message = Message {
            message_type: MessageType::Group,
            message: vec![MessageData::Text(MessageText {
                text: "/remind 30m 提醒 记得吃饭".to_string(),
            })],
            ..Default::default()
        };
        let result = DemoCommandRemind
            .process_message(bot_ctx, &message)
            .await
            .unwrap();
        assert!(result);
    }

    #[tokio::test]
    async fn test_demo_command_remind_no_action() {
        let bot_ctx = BotContextBuilder::new().build().unwrap();
        let message = Message {
            message_type: MessageType::Group,
            message: vec![MessageData::Text(MessageText {
                text: "~remind 60 记得喝水".to_string(),
            })],
            ..Default::default()
        };
        let result = DemoCommandRemind
            .process_message(bot_ctx, &message)
            .await
            .unwrap();
        assert!(result);
    }

    #[tokio::test]
    async fn test_demo_command_calc() {
        let bot_ctx = BotContextBuilder::new().build().unwrap();
        let message = Message {
            message_type: MessageType::Group,
            message: vec![MessageData::Text(MessageText {
                text: "-calc 1 2 3 4 + 10".to_string(),
            })],
            ..Default::default()
        };
        let result = DemoCommandCalc
            .process_message(bot_ctx, &message)
            .await
            .unwrap();
        assert!(result);
    }

    #[tokio::test]
    async fn test_demo_command_calc_multiplication() {
        let bot_ctx = BotContextBuilder::new().build().unwrap();
        let message = Message {
            message_type: MessageType::Group,
            message: vec![MessageData::Text(MessageText {
                text: "/calc 2 3 4 * 24".to_string(),
            })],
            ..Default::default()
        };
        let result = DemoCommandCalc
            .process_message(bot_ctx, &message)
            .await
            .unwrap();
        assert!(result);
    }

    #[tokio::test]
    async fn test_demo_command_tag() {
        let bot_ctx = BotContextBuilder::new().build().unwrap();
        let message = Message {
            message_type: MessageType::Group,
            message: vec![MessageData::Text(MessageText {
                text: "~tag 重要 紧急 工作 user 这是一个重要的工作任务".to_string(),
            })],
            ..Default::default()
        };
        let result = DemoCommandTag
            .process_message(bot_ctx, &message)
            .await
            .unwrap();
        assert!(result);
    }

    #[tokio::test]
    async fn test_demo_command_tag_no_type() {
        let bot_ctx = BotContextBuilder::new().build().unwrap();
        let message = Message {
            message_type: MessageType::Group,
            message: vec![MessageData::Text(MessageText {
                text: "-tag 学习 Rust 编程".to_string(),
            })],
            ..Default::default()
        };
        let result = DemoCommandTag
            .process_message(bot_ctx, &message)
            .await
            .unwrap();
        assert!(result);
    }

    #[tokio::test]
    async fn test_demo_command_echo() {
        let bot_ctx = BotContextBuilder::new().build().unwrap();
        let message = Message {
            message_type: MessageType::Group,
            message: vec![MessageData::Text(MessageText {
                text: "/echo 3 Hello World".to_string(),
            })],
            ..Default::default()
        };
        let result = DemoCommandEcho
            .process_message(bot_ctx, &message)
            .await
            .unwrap();
        assert!(result);
    }

    #[tokio::test]
    async fn test_demo_command_echo_no_count() {
        let bot_ctx = BotContextBuilder::new().build().unwrap();
        let message = Message {
            message_type: MessageType::Group,
            message: vec![MessageData::Text(MessageText {
                text: "~echo 测试消息".to_string(),
            })],
            ..Default::default()
        };
        let result = DemoCommandEcho
            .process_message(bot_ctx, &message)
            .await
            .unwrap();
        assert!(result);
    }

    #[tokio::test]
    async fn test_demo_command_schedule() {
        let bot_ctx = BotContextBuilder::new().build().unwrap();
        let message = Message {
            message_type: MessageType::Group,
            message: vec![MessageData::Text(MessageText {
                text: "-schedule 9 12 18 高 work study 完成项目开发".to_string(),
            })],
            ..Default::default()
        };
        let result = DemoCommandSchedule
            .process_message(bot_ctx, &message)
            .await
            .unwrap();
        assert!(result);
    }

    #[tokio::test]
    async fn test_demo_command_schedule_no_priority() {
        let bot_ctx = BotContextBuilder::new().build().unwrap();
        let message = Message {
            message_type: MessageType::Group,
            message: vec![MessageData::Text(MessageText {
                text: "/schedule 8 20 play rest 休息时间".to_string(),
            })],
            ..Default::default()
        };
        let result = DemoCommandSchedule
            .process_message(bot_ctx, &message)
            .await
            .unwrap();
        assert!(result);
    }
}
