use std::{fmt::Debug, sync::Arc};

use crate::{bot_context::BotContext, event};
use async_trait::async_trait;

#[derive(Debug)]
pub enum Processor {
    Post(Box<dyn PostProcessor>),
    Message(Box<dyn MessageProcessor>),
    Notice(Box<dyn NoticeProcessor>),
    Request(Box<dyn RequestProcessor>),
}

impl Processor {
    pub async fn process(
        &self,
        bot_ctx: Arc<BotContext>,
        post: &event::Post,
    ) -> anyhow::Result<bool> {
        if let (Processor::Message(processor), event::Post::Message(message)) = (self, post) {
            processor.process_message(bot_ctx, message).await
        } else if let (Processor::Notice(processor), event::Post::Notice(notice)) = (self, post) {
            processor.process_notice(bot_ctx, notice).await
        } else if let (Processor::Request(processor), event::Post::Request(request)) = (self, post)
        {
            processor.process_request(bot_ctx, request).await
        } else if let Processor::Post(processor) = self {
            processor.process_post(bot_ctx, post).await
        } else {
            Ok(false)
        }
    }
}

#[async_trait]
pub trait PostProcessor: Send + Sync + Debug {
    async fn process_post(
        &self,
        bot_ctx: Arc<BotContext>,
        post: &event::Post,
    ) -> anyhow::Result<bool>;
}

#[async_trait]
pub trait MessageProcessor: Send + Sync + Debug {
    async fn process_message(
        &self,
        bot_ctx: Arc<BotContext>,
        message: &event::Message,
    ) -> anyhow::Result<bool>;
}

#[async_trait]
pub trait NoticeProcessor: Send + Sync + Debug {
    async fn process_notice(
        &self,
        bot_ctx: Arc<BotContext>,
        event: &event::Notice,
    ) -> anyhow::Result<bool>;
}

#[async_trait]
pub trait RequestProcessor: Send + Sync + Debug {
    async fn process_request(
        &self,
        bot_ctx: Arc<BotContext>,
        request: &event::Request,
    ) -> anyhow::Result<bool>;
}

impl Into<Processor> for Box<dyn PostProcessor> {
    fn into(self) -> Processor {
        Processor::Post(self)
    }
}

impl Into<Processor> for Box<dyn MessageProcessor> {
    fn into(self) -> Processor {
        Processor::Message(self)
    }
}

impl Into<Processor> for Box<dyn NoticeProcessor> {
    fn into(self) -> Processor {
        Processor::Notice(self)
    }
}

impl Into<Processor> for Box<dyn RequestProcessor> {
    fn into(self) -> Processor {
        Processor::Request(self)
    }
}
