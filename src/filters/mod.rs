//! Filters for routing updates (command, text, regex, composition).

mod command;
mod regex;
mod text;

pub use command::command;
pub use regex::regex;
pub use text::text;

use crate::dispatcher::{Context, Filter};

/// Compose two filters with AND: both must pass.
#[derive(Clone)]
pub struct FilterAnd<A, B> {
    pub(crate) a: A,
    pub(crate) b: B,
}

impl<A: Filter, B: Filter> Filter for FilterAnd<A, B> {
    fn check(&self, ctx: &Context) -> bool {
        self.a.check(ctx) && self.b.check(ctx)
    }
}

/// Compose two filters with OR: either can pass.
#[derive(Clone)]
pub struct FilterOr<A, B> {
    pub(crate) a: A,
    pub(crate) b: B,
}

impl<A: Filter, B: Filter> Filter for FilterOr<A, B> {
    fn check(&self, ctx: &Context) -> bool {
        self.a.check(ctx) || self.b.check(ctx)
    }
}

/// Extension trait for Filter to allow .and() and .or().
pub trait FilterExt: Filter + Sized {
    fn and<F: Filter>(self, other: F) -> FilterAnd<Self, F> {
        FilterAnd { a: self, b: other }
    }
    fn or<F: Filter>(self, other: F) -> FilterOr<Self, F> {
        FilterOr { a: self, b: other }
    }
}

impl<T: Filter + Sized> FilterExt for T {}
