//! Shared application / control-plane layer for CLI and local UI.
//! Thin wrappers over Bot / Account — no parallel Telegram logic.

mod bot_service;
mod config;
mod error;
mod paths;
mod redact;

#[cfg(feature = "accounts")]
mod account_service;

pub use bot_service::{BotRecord, BotService, BotStatus};
pub use config::{GramConfig, UiConfig};
pub use error::{ControlError, ExitCode};
pub use paths::GramPaths;
pub use redact::{mask_token, redact_secrets};

#[cfg(feature = "accounts")]
pub use account_service::{
    AccountRecord, AccountService, AccountView, AuthPhase, LoginPrompts, LoginRequest, LoginResult,
};
