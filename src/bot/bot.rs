//! Bot struct with builder and high-level API methods.

use crate::config::{BotConfig, ParseMode};
use crate::core::{ApiClient, TelegramError};
use serde::Serialize;

/// Bot instance: config + API client. Builder via `Bot::new().token(...).workers(...)`.
pub struct Bot {
    pub(crate) config: BotConfig,
    pub(crate) api: ApiClient,
}

impl Bot {
    /// Create a new bot builder.
    pub fn new() -> Self {
        Self {
            config: BotConfig::default(),
            api: ApiClient::new(""),
        }
    }

    /// Set the bot token.
    pub fn token(mut self, token: impl Into<String>) -> Self {
        let token = token.into();
        self.config.token = token.clone();
        self.api = ApiClient::new(token);
        self
    }

    /// Set number of workers for processing updates.
    pub fn workers(mut self, workers: usize) -> Self {
        self.config.workers = workers;
        self
    }

    /// Set default parse mode for messages.
    pub fn parse_mode(mut self, mode: ParseMode) -> Self {
        self.config.parse_mode = mode;
        self
    }

    /// Send a text message to a chat. High-level wrapper for handlers.
    pub async fn send_message(
        &self,
        chat_id: i64,
        text: &str,
    ) -> Result<crate::core::Message, TelegramError> {
        self.send_message_optional(chat_id, text, None).await
    }

    /// Send a text message with optional parse mode.
    pub async fn send_message_optional(
        &self,
        chat_id: i64,
        text: &str,
        parse_mode_override: Option<ParseMode>,
    ) -> Result<crate::core::Message, TelegramError> {
        let parse_mode = parse_mode_override
            .as_ref()
            .or(Some(&self.config.parse_mode))
            .and_then(ParseMode::as_str);
        #[derive(Serialize)]
        struct SendMessage {
            chat_id: i64,
            text: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            parse_mode: Option<String>,
        }
        let body = SendMessage {
            chat_id,
            text: text.to_string(),
            parse_mode: parse_mode.map(String::from),
        };
        self.api.post("sendMessage", &body).await
    }

    /// Answer a callback query (e.g. to remove loading state).
    pub async fn answer_callback_query(
        &self,
        callback_query_id: &str,
        text: Option<&str>,
    ) -> Result<bool, TelegramError> {
        #[derive(Serialize)]
        struct AnswerCallbackQuery<'a> {
            callback_query_id: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            text: Option<&'a str>,
        }
        let body = AnswerCallbackQuery {
            callback_query_id,
            text,
        };
        self.api.post("answerCallbackQuery", &body).await
    }

    /// Run the bot with the given dispatcher (start polling).
    pub async fn run(self, dispatcher: Dispatcher) -> Result<(), TelegramError> {
        crate::runtime::Polling::new(self, dispatcher).start().await
    }
}

impl Default for Bot {
    fn default() -> Self {
        Self::new()
    }
}

// Forward declaration: Dispatcher is in dispatcher module. We need to avoid circular deps.
// Bot::run uses runtime::Polling which uses Dispatcher. So we need to use the type here.
use crate::dispatcher::Dispatcher;
