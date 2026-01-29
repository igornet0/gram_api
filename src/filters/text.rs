//! Text filter: message has non-empty text.

use crate::dispatcher::{Context, Filter};

/// Filter that passes when the message has text.
#[derive(Debug, Clone, Copy)]
pub struct TextFilter;

/// Create a filter that passes when the message has any text.
pub fn text() -> TextFilter {
    TextFilter
}

impl Filter for TextFilter {
    fn check(&self, ctx: &Context) -> bool {
        ctx.message_text().map(|t| !t.is_empty()).unwrap_or(false)
    }
}
