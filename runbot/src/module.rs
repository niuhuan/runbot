use async_trait::async_trait;

use crate::process::Processor;

#[async_trait]
pub trait Module {
    fn id() -> &'static str;

    fn name() -> &'static str;

    fn help() -> &'static str;

    fn processors() -> Vec<Processor>;
}
