//! Context passed to every handler.

use crate::core::Update;
use crate::fsm::{FsmKey, State, Storage};
use crate::Bot;
use std::sync::Arc;

/// Context passed to handlers. Contains the current update, bot reference, and optional FSM storage.
pub struct Context {
    pub update: Update,
    pub bot: Arc<Bot>,
    pub(crate) storage: Option<Arc<dyn Storage>>,
    pub(crate) fsm_key: FsmKey,
}

impl Context {
    pub fn new(
        update: Update,
        bot: Arc<Bot>,
        storage: Option<Arc<dyn Storage>>,
        fsm_key: FsmKey,
    ) -> Self {
        Self {
            update,
            bot,
            storage,
            fsm_key,
        }
    }

    /// Chat ID from message or callback query, if present.
    pub fn chat_id(&self) -> Option<i64> {
        if let Some(ref msg) = self.update.message {
            return Some(msg.chat.id);
        }
        if let Some(ref msg) = self.update.edited_message {
            return Some(msg.chat.id);
        }
        if let Some(ref cq) = self.update.callback_query {
            if let Some(ref msg) = cq.message {
                return Some(msg.chat.id);
            }
        }
        None
    }

    /// User ID from message or callback query, if present.
    pub fn user_id(&self) -> Option<i64> {
        if let Some(ref msg) = self.update.message {
            return msg.from.as_ref().map(|u| u.id);
        }
        if let Some(ref msg) = self.update.edited_message {
            return msg.from.as_ref().map(|u| u.id);
        }
        if let Some(ref cq) = self.update.callback_query {
            return Some(cq.from.id);
        }
        None
    }

    /// Text of the message, if present.
    pub fn message_text(&self) -> Option<&str> {
        self.update
            .message
            .as_ref()
            .and_then(|m| m.text.as_deref())
            .or_else(|| {
                self.update
                    .edited_message
                    .as_ref()
                    .and_then(|m| m.text.as_deref())
            })
    }

    /// Callback query data, if this update is a callback query.
    pub fn callback_data(&self) -> Option<&str> {
        self.update
            .callback_query
            .as_ref()
            .and_then(|cq| cq.data.as_deref())
    }

    /// Get current FSM state for this user/chat, if storage is set.
    pub async fn get_state(&self) -> Option<State> {
        let storage = self.storage.clone()?;
        storage.get(self.fsm_key).await
    }

    /// Set FSM state for this user/chat. Requires storage to be set.
    pub async fn set_state(&self, state: State) -> Result<(), crate::fsm::FsmStorageError> {
        let storage = self
            .storage
            .as_ref()
            .ok_or_else(|| crate::fsm::FsmStorageError::Other(anyhow::anyhow!("no FSM storage")))?;
        storage.set(self.fsm_key, state).await
    }

    /// Clear FSM state for this user/chat.
    pub async fn clear_state(&self) -> Result<(), crate::fsm::FsmStorageError> {
        let storage = self
            .storage
            .as_ref()
            .ok_or_else(|| crate::fsm::FsmStorageError::Other(anyhow::anyhow!("no FSM storage")))?;
        storage.remove(self.fsm_key).await
    }
}
