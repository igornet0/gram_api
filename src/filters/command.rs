//! Command filter: matches /command or /command@botname.

use crate::dispatcher::{Context, Filter};

/// Filter that matches a bot command (e.g. /start or /start@mybot).
#[derive(Debug, Clone)]
pub struct CommandFilter {
    command: String,
}

/// Create a filter that passes when the message text is the given command.
/// Matches /cmd or /cmd@botname (case-insensitive for the command part).
pub fn command(cmd: impl Into<String>) -> CommandFilter {
    CommandFilter {
        command: cmd.into().to_lowercase(),
    }
}

impl Filter for CommandFilter {
    fn check(&self, ctx: &Context) -> bool {
        let text = match ctx.message_text() {
            Some(t) => t.trim(),
            None => return false,
        };
        let text = text.split_whitespace().next().unwrap_or("");
        if !text.starts_with('/') {
            return false;
        }
        let rest = text[1..].split('@').next().unwrap_or("").to_lowercase();
        rest == self.command
    }
}
