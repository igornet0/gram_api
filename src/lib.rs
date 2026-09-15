//! tgram-api: Telegram Bot API for Rust.
//! Enable feature `accounts` for User/TDLib accounts (links TDLib).
//! Enable feature `cli` (default) for the `gram` control-plane binary + local UI.

#[cfg(feature = "accounts")]
pub mod account;
pub mod bot;
pub mod config;
pub mod core;
pub mod dispatcher;
pub mod filters;
pub mod fsm;
pub mod handlers;
pub mod middlewares;
pub mod runtime;

#[cfg(feature = "cli")]
pub mod cli;
#[cfg(feature = "cli")]
pub mod control;
#[cfg(feature = "cli")]
pub mod server;

#[cfg(feature = "accounts")]
pub use account::{Account, AccountError, AccountManager, TdlibCredentials};
pub use bot::Bot;
pub use config::{BotConfig, ParseMode};
pub use core::{ApiClient, TelegramError, Update};
pub use dispatcher::{
    Context, Dispatcher, Filter, Handler, HandlerError, HandlerResult, Middleware, MiddlewareError,
};
pub use filters::{command, regex, text, FilterExt};
pub use fsm::{FsmKey, FsmStorageError, MemoryStorage, State, Storage};
pub use middlewares::{LoggingMiddleware, ThrottleMiddleware};
pub use runtime::Polling;

pub mod prelude;
