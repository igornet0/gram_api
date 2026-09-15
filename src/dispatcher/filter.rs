//! Filter trait for routing updates.

use super::Context;

/// Filter: determines whether a handler should run for the given context.
pub trait Filter: Send + Sync {
    fn check(&self, ctx: &Context) -> bool;
}

/// Filter that always passes (for "any message" handlers).
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct Always;

impl Filter for Always {
    fn check(&self, _ctx: &Context) -> bool {
        true
    }
}
