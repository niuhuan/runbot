//! A QQ bot framework implementing the OneBot 11 protocol, with forward WebSocket
//! client and reverse WebSocket server modes.
//!
//! Callers register processors with [`bot_context::BotContextBuilder`] and build a
//! [`bot_context::BotContext`], then run [`connection::loop_client`] to connect to an
//! OneBot implementation. Alternatively, use [`bot_context::BotServerBuilder`] and
//! [`connection::loop_server`] to accept reverse connections initiated by the OneBot
//! implementation. Both modes require a Tokio runtime; the connection loops are
//! long-running futures that the caller must keep polling.
//!
//! # Minimal flow
//!
//! ```no_run
//! use std::sync::Arc;
//! use runbot::prelude::*;
//!
//! #[processor]
//! async fn echo_hello(
//!     bot_ctx: Arc<BotContext>,
//!     message: &Message,
//! ) -> anyhow::Result<bool> {
//!     if message.raw_message == "hello" {
//!         bot_ctx.send_private_message(message.user_id, "world").await?;
//!     }
//!     Ok(false)
//! }
//!
//! # async fn run() -> runbot::error::Result<()> {
//! let bot_ctx = BotContextBuilder::new()
//!     .url("ws://127.0.0.1:3001")
//!     .add_processor(ECHO_HELLO)
//!     .build()?;
//! loop_client(bot_ctx).await
//! # }
//! # fn main() {}
//! ```
//!
//! # Processors, commands, and modules
//!
//! `#[processor]` supports ordinary event functions and command functions. The ordinary
//! form takes two parameters. The command form appends parameters after
//! `Arc<BotContext>` and `&Message`, in the order the pattern names them:
//!
//! ```rust
//! use std::sync::Arc;
//! use runbot::prelude::*;
//!
//! #[processor(command = "[-|/]mute {seconds:n} {user:n}+")]
//! pub async fn mute_command(
//!     _bot_ctx: Arc<BotContext>,
//!     message: &Message,
//!     seconds: i64,
//!     user: Vec<i64>,
//! ) -> anyhow::Result<bool> {
//!     let _ = (message, seconds, user);
//!     Ok(true)
//! }
//!
//! let _processor: Processor = MUTE_COMMAND.into();
//! ```
//!
//! `#[module]` is applied to `impl Module for ...` and generates a module struct and a
//! static instance. The `processors` attribute joins child processor names with `+`:
//!
//! ```rust
//! use std::sync::Arc;
//! use runbot::prelude::*;
//!
//! #[processor]
//! pub async fn module_child(
//!     _bot_ctx: Arc<BotContext>,
//!     _message: &Message,
//! ) -> anyhow::Result<bool> {
//!     Ok(false)
//! }
//!
//! #[module(name = "ExampleMod", help = "Example module", processors = "module_child")]
//! impl Module for ExampleMod {}
//!
//! let _processor: Processor = EXAMPLE_MOD.into();
//! ```
//!
//! `ParseJson` generates `parse(&serde_json::Value)` for a type that already implements
//! `Deserialize`; `UnknownTypeSerde` maps an enum to known strings plus `Unknown(String)`:
//!
//! ```rust
//! use runbot::error::Result;
//! use runbot::re_export::serde_json;
//!
//! #[derive(serde_derive::Deserialize, runbot_codegen::ParseJson)]
//! struct ParseExample {
//!     value: i64,
//! }
//!
//! #[derive(runbot_codegen::UnknownTypeSerde)]
//! enum ExampleType {
//!     Known,
//!     Unknown(String),
//! }
//!
//! # fn main() -> Result<()> {
//! let parsed = ParseExample::parse(&serde_json::json!({"value": 1}))?;
//! assert_eq!(parsed.value, 1);
//!
//! let known: ExampleType = serde_json::from_str("\"known\"")?;
//! assert!(matches!(known, ExampleType::Known));
//! let other: ExampleType = serde_json::from_str("\"other\"")?;
//! assert!(matches!(other, ExampleType::Unknown(value) if value == "other"));
//! # Ok(())
//! # }
//! ```
//!
//! See the `runbot-codegen` documentation for the complete input restrictions, generated
//! item names, and compile-time diagnostics.
//!
//! # Module map
//!
//! - [`bot_context`]: connection context, processor list, and processor registration.
//! - [`connection`]: forward and reverse WebSocket connection loops.
//! - [`event`]: OneBot events, API responses, message segments, and message chains.
//! - [`process`]: processor traits and the unified dispatch enum.
//! - [`module`]: trait for nestable modules.
//! - [`command`]: tokenization and argument extraction used by the command-pattern macro.
//! - [`onebot11_api`]: OneBot API wrappers and response types exposed as inherent methods
//!   on [`bot_context::BotContext`].
//! - [`error`]: unified error type and [`error::Result`].
//! - [`common`]: field deserialization helpers.
//! - [`re_export`]: dependency re-exports used by runbot and generated macro code.
//! - [`prelude`]: common imports.
//!
//! # Runtime and deployment
//!
//! The connection loops run on a Tokio runtime and must be called from a Tokio context.
//! Before a connection is established, [`bot_context::BotContext`] methods that send
//! requests return [`error::Error::StateError`]. Business code should therefore run
//! after the connection is ready, rather than assuming connectivity before
//! [`connection::loop_client`] returns.
//!
//! This crate does not implement TLS. Use a `wss://` reverse proxy or a trusted TLS
//! terminator when encryption is required. Access tokens are sent in the
//! `Authorization: Bearer` header and are never placed in the URL, but Bearer
//! authentication by itself does not encrypt the transport.

pub mod bot_context;
pub mod command;
pub mod common;
pub mod connection;
pub mod error;
pub mod event;
pub mod module;
pub mod onebot11_api;
pub mod process;
pub mod re_export;

/// Common imports for typical applications.
///
/// This module re-exports the connection context, connection loops, event types, module
/// and processor traits, and the `#[processor]` and `#[module]` attribute macros.
/// `BotContext`'s OneBot API methods are re-exported through `onebot11_api` and can be
/// called directly after importing this module.
pub mod prelude {
    pub use crate::bot_context::*;
    pub use crate::connection::*;
    pub use crate::event::*;
    pub use crate::module::*;
    pub use crate::onebot11_api::*;
    pub use crate::process::*;
    pub use runbot_codegen::{module, processor};
}
