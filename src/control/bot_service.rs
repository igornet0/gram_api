//! Bot registry + thin Bot API checks (getMe / send via library).

use std::fs;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::bot::Bot;
use crate::core::ApiClient;

use super::config::GramConfig;
use super::paths::GramPaths;
use super::redact::mask_token;
use super::ControlError;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BotRecord {
    pub name: String,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub desired_running: bool,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct BotStatus {
    pub name: String,
    pub username: Option<String>,
    pub desired_running: bool,
    pub token_masked: String,
    pub reachable: Option<bool>,
}

pub struct BotService {
    paths: GramPaths,
    #[allow(dead_code)]
    config: GramConfig,
    lock: Mutex<()>,
}

impl BotService {
    pub fn new(paths: GramPaths, config: GramConfig) -> Result<Self, ControlError> {
        paths.ensure()?;
        Ok(Self {
            paths,
            config,
            lock: Mutex::new(()),
        })
    }

    pub fn list(&self) -> Result<Vec<BotRecord>, ControlError> {
        let dir = self.paths.bots();
        let mut out = Vec::new();
        if !dir.exists() {
            return Ok(out);
        }
        for entry in fs::read_dir(&dir).map_err(|e| ControlError::Io(e.to_string()))? {
            let entry = entry.map_err(|e| ControlError::Io(e.to_string()))?;
            let meta = entry.path().join("bot.toml");
            if meta.is_file() {
                let raw = fs::read_to_string(&meta).map_err(|e| ControlError::Io(e.to_string()))?;
                let rec: BotRecord =
                    toml::from_str(&raw).map_err(|e| ControlError::Config(e.to_string()))?;
                out.push(rec);
            }
        }
        out.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(out)
    }

    pub fn add(&self, name: &str, token: &str) -> Result<BotRecord, ControlError> {
        let _g = self.lock.lock().unwrap();
        let name = name.trim();
        if name.is_empty() {
            return Err(ControlError::Invalid("bot name required".into()));
        }
        if token.trim().is_empty() {
            return Err(ControlError::Invalid("bot token required".into()));
        }
        let dir = self.paths.bot(name);
        fs::create_dir_all(&dir).map_err(|e| ControlError::Io(e.to_string()))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&dir, fs::Permissions::from_mode(0o700));
        }
        let token_path = self.paths.bot_token_file(name);
        fs::write(&token_path, token.trim()).map_err(|e| ControlError::Io(e.to_string()))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&token_path, fs::Permissions::from_mode(0o600));
        }
        let rec = BotRecord {
            name: name.to_string(),
            username: None,
            desired_running: false,
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        self.write_meta(&rec)?;
        Ok(rec)
    }

    pub fn remove(&self, name: &str) -> Result<(), ControlError> {
        let _g = self.lock.lock().unwrap();
        let dir = self.paths.bot(name);
        if !dir.exists() {
            return Err(ControlError::NotFound(format!("bot `{name}` not found")));
        }
        fs::remove_dir_all(&dir).map_err(|e| ControlError::Io(e.to_string()))?;
        Ok(())
    }

    pub fn load_token(&self, name: &str) -> Result<String, ControlError> {
        let path = self.paths.bot_token_file(name);
        fs::read_to_string(&path)
            .map(|s| s.trim().to_string())
            .map_err(|_| ControlError::NotFound(format!("bot `{name}` not found")))
    }

    pub async fn status(&self, name: &str) -> Result<BotStatus, ControlError> {
        let rec = self.load_meta(name)?;
        let token = self.load_token(name)?;
        let reachable = match self.probe(&token).await {
            Ok(_) => Some(true),
            Err(_) => Some(false),
        };
        Ok(BotStatus {
            name: rec.name,
            username: rec.username,
            desired_running: rec.desired_running,
            token_masked: mask_token(&token),
            reachable,
        })
    }

    pub async fn start(&self, name: &str) -> Result<BotStatus, ControlError> {
        let token = self.load_token(name)?;
        let me = self.probe(&token).await?;
        let mut rec = self.load_meta(name)?;
        rec.desired_running = true;
        if let Some(u) = me {
            rec.username = Some(u);
        }
        self.write_meta(&rec)?;
        self.status(name).await
    }

    pub async fn stop(&self, name: &str) -> Result<BotStatus, ControlError> {
        let mut rec = self.load_meta(name)?;
        rec.desired_running = false;
        self.write_meta(&rec)?;
        self.status(name).await
    }

    /// Build a library [`Bot`] from stored token.
    pub fn open_bot(&self, name: &str) -> Result<Bot, ControlError> {
        let token = self.load_token(name)?;
        Ok(Bot::new().token(token))
    }

    async fn probe(&self, token: &str) -> Result<Option<String>, ControlError> {
        let client = ApiClient::new(token);
        #[derive(serde::Deserialize)]
        struct User {
            username: Option<String>,
        }
        let user: User = client.get("getMe").await?;
        Ok(user.username)
    }

    fn load_meta(&self, name: &str) -> Result<BotRecord, ControlError> {
        let path = self.paths.bot_meta(name);
        let raw = fs::read_to_string(&path)
            .map_err(|_| ControlError::NotFound(format!("bot `{name}` not found")))?;
        toml::from_str(&raw).map_err(|e| ControlError::Config(e.to_string()))
    }

    fn write_meta(&self, rec: &BotRecord) -> Result<(), ControlError> {
        let path = self.paths.bot_meta(&rec.name);
        let raw = toml::to_string_pretty(rec).map_err(|e| ControlError::Config(e.to_string()))?;
        fs::write(&path, raw).map_err(|e| ControlError::Io(e.to_string()))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o600));
        }
        Ok(())
    }
}
