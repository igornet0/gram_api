//! Logging middleware: log incoming updates.

use crate::dispatcher::{Context, Middleware, MiddlewareError};
use async_trait::async_trait;

/// Middleware that logs each update (update_id and type).
pub struct LoggingMiddleware;

impl LoggingMiddleware {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LoggingMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Middleware for LoggingMiddleware {
    async fn before(&self, ctx: &mut Context) -> Result<(), MiddlewareError> {
        let mut kind = String::new();
        if ctx.update.message.is_some() {
            kind.push_str("message");
        }
        if ctx.update.edited_message.is_some() {
            if !kind.is_empty() {
                kind.push(',');
            }
            kind.push_str("edited_message");
        }
        if ctx.update.callback_query.is_some() {
            if !kind.is_empty() {
                kind.push(',');
            }
            kind.push_str("callback_query");
        }
        if kind.is_empty() {
            kind = "other".to_string();
        }
        eprintln!("[tgram] update_id={} kind={}", ctx.update.update_id, kind);
        Ok(())
    }
}
