//! Error types for Telegram API client.

use thiserror::Error;

/// Errors returned by the Telegram Bot API or the HTTP client.
#[derive(Debug, Error)]
pub enum TelegramError {
    /// API returned an error (e.g. `ok: false`).
    #[error("Telegram API error: {description} (code: {error_code:?})")]
    Api {
        description: String,
        error_code: Option<i32>,
    },

    /// Network or HTTP client error.
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    /// JSON decode error.
    #[error("Decode error: {0}")]
    Decode(#[from] serde_json::Error),
}
