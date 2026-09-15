//! Never log or print secrets.

use regex::Regex;
use std::sync::OnceLock;

pub fn mask_token(token: &str) -> String {
    if token.len() <= 8 {
        return "********".into();
    }
    format!(
        "{}…{}",
        &token[..4],
        &token[token.len().saturating_sub(4)..]
    )
}

/// Redact common secret substrings from log / error lines.
pub fn redact_secrets(text: &str) -> String {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| {
        Regex::new(concat!(
            r"(?i)((?:password|api_hash|token|bot_token|auth_code|2fa)\s*[=:]\s*)(\S+)",
            r"|",
            r"(\d{8,10}:[A-Za-z0-9_-]{20,})",
        ))
        .expect("redact regex")
    });
    re.replace_all(text, |caps: &regex::Captures| {
        if let Some(m) = caps.get(3) {
            mask_token(m.as_str())
        } else {
            format!("{}[redacted]", &caps[1])
        }
    })
    .into_owned()
}
