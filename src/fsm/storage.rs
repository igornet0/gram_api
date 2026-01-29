//! FSM storage trait.

use async_trait::async_trait;

/// Key for FSM state: (user_id, chat_id). Use (user_id, 0) for user-only state.
pub type FsmKey = (i64, i64);

/// State value: JSON-serializable. Users can use their own enum and serialize to Value.
pub type State = serde_json::Value;

/// Storage for FSM state (in-memory, Redis, etc.).
#[async_trait]
pub trait Storage: Send + Sync {
    /// Get current state for the key, if any.
    async fn get(&self, key: FsmKey) -> Option<State>;

    /// Set state for the key.
    async fn set(&self, key: FsmKey, state: State) -> Result<(), FsmStorageError>;

    /// Remove state for the key.
    async fn remove(&self, key: FsmKey) -> Result<(), FsmStorageError>;
}

/// Errors from FSM storage.
#[derive(Debug, thiserror::Error)]
pub enum FsmStorageError {
    #[error("Storage error: {0}")]
    Other(#[from] anyhow::Error),
}
