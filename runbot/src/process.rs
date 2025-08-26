use std::{fmt::Debug, sync::Arc};

use crate::{event, connection::BotContext};
use async_trait::async_trait;


#[async_trait]
pub trait MessageProcessor: Send + Sync + Debug {
    async fn process_message(&self, bot_ctx: Arc<BotContext>, message: Arc<event::Message>) -> anyhow::Result<bool>;
}
