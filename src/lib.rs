//! tgram-api: aiogram-like Telegram Bot API library for Rust.

pub mod config;
pub mod core;
mod bot;
mod dispatcher;
mod filters;
mod fsm;
mod handlers;
mod middlewares;
mod runtime;

pub use bot::Bot;
pub use config::{BotConfig, ParseMode};
pub use core::{ApiClient, TelegramError, Update};
pub use dispatcher::{
    Context, Dispatcher, Filter, Handler, HandlerError, HandlerResult, Middleware, MiddlewareError,
};
pub use filters::{command, regex, text, FilterExt};
pub use middlewares::{LoggingMiddleware, ThrottleMiddleware};
pub use fsm::{FsmKey, FsmStorageError, MemoryStorage, State, Storage};
pub use runtime::Polling;

pub mod prelude;
