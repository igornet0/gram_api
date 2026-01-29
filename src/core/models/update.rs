//! Update model.

use super::{CallbackQuery, Message};
use serde::{Deserialize, Serialize};

/// Represents an incoming update.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Update {
    pub update_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edited_message: Option<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callback_query: Option<CallbackQuery>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inline_query: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chosen_inline_result: Option<serde_json::Value>,
}
