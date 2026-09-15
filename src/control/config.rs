//! Gram configuration (CLI > env > config.toml > defaults).

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::paths::GramPaths;
use super::ControlError;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GramConfig {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub ui: UiConfig,
    #[serde(default)]
    pub telegram: TelegramAppConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GeneralConfig {
    #[serde(default = "default_data_hint")]
    pub data_dir: String,
}

fn default_data_hint() -> String {
    "~/.gram".into()
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            data_dir: default_data_hint(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UiConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

fn default_host() -> String {
    "127.0.0.1".into()
}
fn default_port() -> u16 {
    8788
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
        }
    }
}

/// Application credentials for TDLib (not a user session).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TelegramAppConfig {
    pub api_id: Option<i32>,
    pub api_hash: Option<String>,
    #[serde(default)]
    pub use_test_dc: bool,
}

impl GramConfig {
    pub fn load(paths: &GramPaths) -> Result<Self, ControlError> {
        let file = paths.config_file();
        if !file.exists() {
            let cfg = Self::default();
            cfg.save(paths)?;
            return Ok(cfg);
        }
        let raw = fs::read_to_string(&file).map_err(|e| ControlError::Io(e.to_string()))?;
        let mut cfg: Self =
            toml::from_str(&raw).map_err(|e| ControlError::Config(e.to_string()))?;
        // Env overrides
        if let Ok(id) = std::env::var("TELEGRAM_API_ID") {
            if let Ok(v) = id.parse() {
                cfg.telegram.api_id = Some(v);
            }
        }
        if let Ok(hash) = std::env::var("TELEGRAM_API_HASH") {
            if !hash.is_empty() {
                cfg.telegram.api_hash = Some(hash);
            }
        }
        if let Ok(port) = std::env::var("GRAM_UI_PORT") {
            if let Ok(v) = port.parse() {
                cfg.ui.port = v;
            }
        }
        if let Ok(host) = std::env::var("GRAM_UI_HOST") {
            if !host.is_empty() {
                cfg.ui.host = host;
            }
        }
        Ok(cfg)
    }

    pub fn save(&self, paths: &GramPaths) -> Result<(), ControlError> {
        paths.ensure()?;
        let raw = toml::to_string_pretty(self).map_err(|e| ControlError::Config(e.to_string()))?;
        let file = paths.config_file();
        fs::write(&file, raw).map_err(|e| ControlError::Io(e.to_string()))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&file, fs::Permissions::from_mode(0o600));
        }
        Ok(())
    }

    pub fn resolve_data_dir(cli_data: Option<PathBuf>) -> PathBuf {
        if let Some(p) = cli_data {
            return expand_tilde(p);
        }
        if let Ok(env) = std::env::var("GRAM_DATA_DIR") {
            return expand_tilde(PathBuf::from(env));
        }
        GramPaths::default_root()
    }
}

fn expand_tilde(path: PathBuf) -> PathBuf {
    let s = path.to_string_lossy();
    if let Some(rest) = s.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest);
        }
    }
    path
}
