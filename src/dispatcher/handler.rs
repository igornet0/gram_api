//! Handler trait for processing updates.

use crate::dispatcher::Context;
use async_trait::async_trait;
use thiserror::Error;

/// Result type for handlers.
pub type HandlerResult = Result<(), HandlerError>;

/// Errors that can occur in a handler.
#[derive(Debug, Error)]
pub enum HandlerError {
    #[error("Telegram API error: {0}")]
    Telegram(#[from] crate::core::TelegramError),
    #[error("FSM storage error: {0}")]
    Fsm(#[from] crate::fsm::FsmStorageError),
    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}

/// Handler trait: process a context (update + bot).
#[async_trait]
pub trait Handler: Send + Sync {
    async fn handle(&self, ctx: Context) -> HandlerResult;
}

#[async_trait]
impl<F, Fut> Handler for F
where
    F: Fn(Context) -> Fut + Send + Sync,
    Fut: std::future::Future<Output = HandlerResult> + Send,
{
    async fn handle(&self, ctx: Context) -> HandlerResult {
        (self)(ctx).await
    }
}
