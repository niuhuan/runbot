pub mod bot_context;
pub mod connection;
pub mod error;
pub mod event;
pub mod process;
pub mod re_export;

pub mod prelude {
    pub use crate::bot_context::*;
    pub use crate::connection::*;
    pub use crate::event::*;
    pub use crate::process::*;
    pub use runbot_codegen::processor;
}
