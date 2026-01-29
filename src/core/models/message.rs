//! Message model.

use super::{Chat, User};
use serde::{Deserialize, Serialize};

/// Represents a message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub message_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<User>,
    pub date: i64,
    pub chat: Chat,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to_message: Option<Box<Message>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<super::keyboard::InlineKeyboardMarkup>,
}
