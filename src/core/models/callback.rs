//! Callback query model.

use super::{Message, User};
use serde::{Deserialize, Serialize};

/// Represents an incoming callback query from a callback button.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallbackQuery {
    pub id: String,
    pub from: User,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inline_message_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chat_instance: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
}
