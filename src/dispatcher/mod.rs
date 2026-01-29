//! Dispatcher: routing updates to handlers with filters.

mod context;
mod filter;
mod handler;
mod router;

pub use context::Context;
pub use filter::Filter;
pub use handler::{Handler, HandlerError, HandlerResult};
pub use router::{Dispatcher, Middleware, MiddlewareError};
