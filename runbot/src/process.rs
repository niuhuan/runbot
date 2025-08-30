use std::{fmt::Debug, sync::Arc};

use crate::{
    bot_context::BotContext,
    event::{self, Post},
};
use async_trait::async_trait;

#[derive(Debug)]
pub enum Processor {
    Post(Box<dyn PostProcessor>),
    Message(Box<dyn MessageProcessor>),
    Notice(Box<dyn NoticeProcessor>),
    Request(Box<dyn RequestProcessor>),
    Module(Box<dyn ModuleProcessor>),
}

impl Processor {
    pub fn id(&self) -> &'static str {
        match self {
            Processor::Post(processor) => processor.id(),
            Processor::Message(processor) => processor.id(),
            Processor::Notice(processor) => processor.id(),
            Processor::Request(processor) => processor.id(),
            Processor::Module(processor) => processor.id(),
        }
    }

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
        } else if let Processor::Module(processor) = self {
            processor.process_post(bot_ctx, post).await
        } else {
            Ok(false)
        }
    }
}

#[async_trait]
pub trait PostProcessor: Send + Sync + Debug {
    fn id(&self) -> &'static str;
    async fn process_post(
        &self,
        bot_ctx: Arc<BotContext>,
        post: &event::Post,
    ) -> anyhow::Result<bool>;
}

#[async_trait]
pub trait MessageProcessor: Send + Sync + Debug {
    fn id(&self) -> &'static str;
    async fn process_message(
        &self,
        bot_ctx: Arc<BotContext>,
        message: &event::Message,
    ) -> anyhow::Result<bool>;
}

#[async_trait]
pub trait NoticeProcessor: Send + Sync + Debug {
    fn id(&self) -> &'static str;
    async fn process_notice(
        &self,
        bot_ctx: Arc<BotContext>,
        event: &event::Notice,
    ) -> anyhow::Result<bool>;
}

#[async_trait]
pub trait RequestProcessor: Send + Sync + Debug {
    fn id(&self) -> &'static str;

    async fn process_request(
        &self,
        bot_ctx: Arc<BotContext>,
        request: &event::Request,
    ) -> anyhow::Result<bool>;
}

#[async_trait]
pub trait ModuleProcessor: Send + Sync + Debug {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn help(&self) -> &'static str;
    async fn process_post(
        &self,
        bot_ctx: Arc<BotContext>,
        post: &event::Post,
    ) -> anyhow::Result<bool>;
    fn processors(&self) -> Arc<Vec<Processor>>;
}

#[derive(Debug)]
pub struct ProcessModule {
    pub id: &'static str,
    pub name: &'static str,
    pub help: &'static str,
    pub processors: Arc<Vec<Processor>>,
}

#[async_trait]
impl ModuleProcessor for ProcessModule {
    fn id(&self) -> &'static str {
        self.id
    }
    fn name(&self) -> &'static str {
        self.name
    }
    fn help(&self) -> &'static str {
        self.help
    }
    async fn process_post(
        &self,
        bot_ctx: Arc<BotContext>,
        post: &event::Post,
    ) -> anyhow::Result<bool> {
        loop_processors(bot_ctx, self.processors.iter(), post).await
    }
    fn processors(&self) -> Arc<Vec<Processor>> {
        self.processors.clone()
    }
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

impl Into<Processor> for Box<dyn ModuleProcessor> {
    fn into(self) -> Processor {
        Processor::Module(self)
    }
}

pub(crate) async fn loop_processors(
    bot_ctx: Arc<BotContext>,
    processor_iter: core::slice::Iter<'_, Processor>,
    post: &Post,
) -> anyhow::Result<bool> {
    for processor in processor_iter {
        let processe_result = processor.process(bot_ctx.clone(), post).await;
        match processe_result {
            Ok(b) => {
                if b {
                    tracing::debug!(
                        "post processed, id : {:?} , post : {:?}",
                        processor.id(),
                        post
                    );
                    return Ok(b);
                }
            }
            Err(err) => {
                tracing::error!(
                    "processor error, id: {:?} , post {:?} error: {:?}",
                    processor.id(),
                    post,
                    err
                );
                return Err(err);
            }
        }
    }
    Ok(false)
}
