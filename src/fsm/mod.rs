//! FSM (Finite State Machine): state storage and context access.

mod memory;
mod storage;

pub use memory::MemoryStorage;
pub use storage::{FsmKey, FsmStorageError, State, Storage};
