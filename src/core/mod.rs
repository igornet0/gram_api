//! Core: HTTP client, errors, and Telegram models.

mod client;
mod error;
pub mod models;

pub use client::{ApiClient, ApiResponse};
pub use error::TelegramError;
pub use models::*;
