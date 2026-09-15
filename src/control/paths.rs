//! Data directory layout under `~/.gram` (or `--data`).

use std::fs;
use std::path::{Path, PathBuf};

use super::ControlError;

#[derive(Clone, Debug)]
pub struct GramPaths {
    root: PathBuf,
}

impl GramPaths {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn default_root() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".gram")
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn config_file(&self) -> PathBuf {
        self.root.join("config.toml")
    }

    pub fn accounts(&self) -> PathBuf {
        self.root.join("accounts")
    }

    pub fn account(&self, key: &str) -> PathBuf {
        self.accounts().join(sanitize_key(key))
    }

    pub fn account_meta(&self, key: &str) -> PathBuf {
        self.account(key).join("account.toml")
    }

    pub fn account_session(&self, key: &str) -> PathBuf {
        self.account(key).join("session")
    }

    pub fn bots(&self) -> PathBuf {
        self.root.join("bots")
    }

    pub fn bot(&self, name: &str) -> PathBuf {
        self.bots().join(sanitize_key(name))
    }

    pub fn bot_meta(&self, name: &str) -> PathBuf {
        self.bot(name).join("bot.toml")
    }

    pub fn bot_token_file(&self, name: &str) -> PathBuf {
        self.bot(name).join("token")
    }

    pub fn logs(&self) -> PathBuf {
        self.root.join("logs")
    }

    pub fn ui(&self) -> PathBuf {
        self.root.join("ui")
    }

    /// Ensure directory tree exists with restrictive permissions on Unix.
    pub fn ensure(&self) -> Result<(), ControlError> {
        for dir in [
            self.root.clone(),
            self.accounts(),
            self.bots(),
            self.logs(),
            self.ui(),
        ] {
            fs::create_dir_all(&dir).map_err(|e| ControlError::Io(e.to_string()))?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = fs::set_permissions(&dir, fs::Permissions::from_mode(0o700));
            }
        }
        Ok(())
    }
}

pub fn sanitize_key(raw: &str) -> String {
    let digits: String = raw.chars().filter(|c| c.is_ascii_digit()).collect();
    if !digits.is_empty() {
        return digits;
    }
    raw.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(feature = "accounts")]
pub fn normalize_phone(phone: &str) -> String {
    let trimmed = phone.trim();
    if trimmed.starts_with('+') {
        format!(
            "+{}",
            trimmed
                .chars()
                .skip(1)
                .filter(|c| c.is_ascii_digit())
                .collect::<String>()
        )
    } else {
        format!(
            "+{}",
            trimmed
                .chars()
                .filter(|c| c.is_ascii_digit())
                .collect::<String>()
        )
    }
}
