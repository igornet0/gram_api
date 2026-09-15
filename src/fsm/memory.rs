//! In-memory FSM storage.

use super::storage::{FsmKey, FsmStorageError, State, Storage};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::RwLock;

/// In-memory storage for FSM state (HashMap + RwLock).
#[derive(Default)]
pub struct MemoryStorage {
    map: RwLock<HashMap<FsmKey, State>>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl Storage for MemoryStorage {
    async fn get(&self, key: FsmKey) -> Option<State> {
        self.map.read().unwrap().get(&key).cloned()
    }

    async fn set(&self, key: FsmKey, state: State) -> Result<(), FsmStorageError> {
        self.map.write().unwrap().insert(key, state);
        Ok(())
    }

    async fn remove(&self, key: FsmKey) -> Result<(), FsmStorageError> {
        self.map.write().unwrap().remove(&key);
        Ok(())
    }
}
