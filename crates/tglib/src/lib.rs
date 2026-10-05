//! tglib — Telegram User / TDLib + Bot API library.
//!
//! Public entry: `use tglib::{TelegramClient, TdlibAdapter, bot::BotClient, ...}`.
//! TDLib `@type` names never leak through this crate.

pub use tglib_client::*;
pub use tglib_core::*;
pub use tglib_tdlib::*;

/// Telegram Bot API HTTP client (`getMe`, `sendMessage`, …).
pub mod bot {
    pub use tglib_bot::*;
}
