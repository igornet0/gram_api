//! Bot configuration.

/// Parse mode for message text (HTML or Markdown).
#[derive(Debug, Clone, Default)]
pub enum ParseMode {
    #[default]
    None,
    Html,
    Markdown,
    MarkdownV2,
}

impl ParseMode {
    pub fn as_str(&self) -> Option<&'static str> {
        match self {
            ParseMode::None => None,
            ParseMode::Html => Some("HTML"),
            ParseMode::Markdown => Some("Markdown"),
            ParseMode::MarkdownV2 => Some("MarkdownV2"),
        }
    }
}

/// Bot configuration (token, workers, parse mode, etc.).
#[derive(Debug, Clone)]
pub struct BotConfig {
    pub token: String,
    pub workers: usize,
    pub parse_mode: ParseMode,
}

impl Default for BotConfig {
    fn default() -> Self {
        Self {
            token: String::new(),
            workers: 4,
            parse_mode: ParseMode::default(),
        }
    }
}
