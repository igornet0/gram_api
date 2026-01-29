//! Dispatcher and router: register handlers and process updates.

use crate::dispatcher::{Context, Filter, Handler, HandlerResult};
use crate::fsm::{FsmKey, Storage};
use crate::Bot;
use std::sync::Arc;
use async_trait::async_trait;

/// A registered route: filter + handler.
struct Route {
    filter: Box<dyn Filter>,
    handler: Box<dyn Handler>,
}

/// Dispatcher holds message handlers and callback handlers, runs middleware, and processes updates.
pub struct Dispatcher {
    message_routes: Vec<Route>,
    callback_routes: Vec<Route>,
    middlewares: Vec<Box<dyn Middleware>>,
    storage: Option<Arc<dyn Storage>>,
}

/// Middleware runs before (and optionally after) handler. Can short-circuit by returning Err.
#[async_trait]
pub trait Middleware: Send + Sync {
    async fn before(&self, ctx: &mut Context) -> Result<(), MiddlewareError>;
}

/// Errors from middleware (e.g. throttle).
#[derive(Debug, thiserror::Error)]
pub enum MiddlewareError {
    #[error("Middleware error: {0}")]
    Other(#[from] anyhow::Error),
}

#[async_trait]
impl<F, Fut> Middleware for F
where
    F: Fn(&mut Context) -> Fut + Send + Sync,
    Fut: std::future::Future<Output = Result<(), MiddlewareError>> + Send,
{
    async fn before(&self, ctx: &mut Context) -> Result<(), MiddlewareError> {
        (self)(ctx).await
    }
}

fn fsm_key_from_update(update: &crate::core::Update) -> FsmKey {
    let (user_id, chat_id) = (
        update.message.as_ref().and_then(|m| m.from.as_ref().map(|u| u.id))
            .or_else(|| update.edited_message.as_ref().and_then(|m| m.from.as_ref().map(|u| u.id)))
            .or_else(|| update.callback_query.as_ref().map(|cq| cq.from.id))
            .unwrap_or(0),
        update.message.as_ref().map(|m| m.chat.id)
            .or_else(|| update.edited_message.as_ref().map(|m| m.chat.id))
            .or_else(|| update.callback_query.as_ref().and_then(|cq| cq.message.as_ref().map(|m| m.chat.id)))
            .unwrap_or(0),
    );
    (user_id, chat_id)
}

impl Dispatcher {
    pub fn new() -> Self {
        Self {
            message_routes: Vec::new(),
            callback_routes: Vec::new(),
            middlewares: Vec::new(),
            storage: None,
        }
    }

    /// Set FSM storage for state (e.g. MemoryStorage::new()).
    pub fn with_storage(mut self, storage: impl Storage + 'static) -> Self {
        self.storage = Some(Arc::new(storage));
        self
    }

    /// Add a middleware (runs in order before handlers).
    pub fn middleware(mut self, m: impl Middleware + 'static) -> Self {
        self.middlewares.push(Box::new(m));
        self
    }

    /// Register a handler for message updates with a filter.
    pub fn message(mut self, filter: impl Filter + 'static, handler: impl Handler + 'static) -> Self {
        self.message_routes.push(Route {
            filter: Box::new(filter),
            handler: Box::new(handler),
        });
        self
    }

    /// Register a handler for callback query updates with a filter.
    pub fn callback(
        mut self,
        filter: impl Filter + 'static,
        handler: impl Handler + 'static,
    ) -> Self {
        self.callback_routes.push(Route {
            filter: Box::new(filter),
            handler: Box::new(handler),
        });
        self
    }

    /// Process a single update: run middlewares, then match first fitting handler.
    pub async fn process(&self, update: Update, bot: Arc<Bot>) -> HandlerResult {
        let fsm_key = fsm_key_from_update(&update);
        let mut ctx = Context::new(update.clone(), bot, self.storage.clone(), fsm_key);

        for m in &self.middlewares {
            m.before(&mut ctx).await.map_err(|e| {
                crate::dispatcher::HandlerError::Other(anyhow::anyhow!("{}", e))
            })?;
        }

        if update.callback_query.is_some() {
            for route in &self.callback_routes {
                if route.filter.check(&ctx) {
                    return route.handler.handle(ctx).await;
                }
            }
        }

        if update.message.is_some() || update.edited_message.is_some() {
            for route in &self.message_routes {
                if route.filter.check(&ctx) {
                    return route.handler.handle(ctx).await;
                }
            }
        }

        Ok(())
    }
}

impl Default for Dispatcher {
    fn default() -> Self {
        Self::new()
    }
}

use crate::core::Update;
