//! Regex filter: message text matches a regex.

use crate::dispatcher::{Context, Filter};
use regex::Regex;

/// Filter that passes when the message text matches the given regex.
#[derive(Clone)]
pub struct RegexFilter {
    regex: Regex,
}

/// Create a filter that passes when the message text matches the regex.
pub fn regex(pattern: &str) -> Result<RegexFilter, regex::Error> {
    Regex::new(pattern).map(|regex| RegexFilter { regex })
}

impl Filter for RegexFilter {
    fn check(&self, ctx: &Context) -> bool {
        ctx.message_text()
            .map(|t| self.regex.is_match(t))
            .unwrap_or(false)
    }
}
