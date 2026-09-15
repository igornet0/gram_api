//! Account registry + auth flows over library [`Account`] (TDLib).

use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::account::{Account, TdlibCredentials, TelegramAccountId, TelegramAccountStatus};

use super::config::GramConfig;
use super::paths::{normalize_phone, sanitize_key, GramPaths};
use super::ControlError;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccountRecord {
    pub key: String,
    pub phone: String,
    pub account_id: String,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct AccountView {
    pub key: String,
    pub phone: String,
    pub status: String,
    pub display_name: Option<String>,
    pub username: Option<String>,
    pub data_dir: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuthPhase {
    PhoneRequired,
    CodeRequired,
    PasswordRequired,
    RegistrationRequired,
    Authorized,
    Error,
}

#[derive(Clone, Debug)]
pub struct LoginRequest {
    pub phone: String,
    pub code: Option<String>,
    pub password: Option<String>,
    pub first_name: Option<String>,
}

/// How to obtain missing auth secrets.
pub enum LoginPrompts<'a> {
    None,
    Interactive {
        code: &'a mut (dyn FnMut() -> Result<String, ControlError> + Send),
        password: &'a mut (dyn FnMut() -> Result<String, ControlError> + Send),
        first_name: &'a mut (dyn FnMut() -> Result<String, ControlError> + Send),
    },
}

#[derive(Clone, Debug)]
pub struct LoginResult {
    pub key: String,
    pub phase: AuthPhase,
    pub message: String,
    pub view: Option<AccountView>,
}

pub struct AccountService {
    paths: GramPaths,
    config: GramConfig,
    /// Live clients held by UI / long-running start.
    live: Mutex<HashMap<String, Arc<Account>>>,
}

impl AccountService {
    pub fn new(paths: GramPaths, config: GramConfig) -> Result<Self, ControlError> {
        paths.ensure()?;
        Ok(Self {
            paths,
            config,
            live: Mutex::new(HashMap::new()),
        })
    }

    pub fn list(&self) -> Result<Vec<AccountRecord>, ControlError> {
        let dir = self.paths.accounts();
        let mut out = Vec::new();
        if !dir.exists() {
            return Ok(out);
        }
        for entry in fs::read_dir(&dir).map_err(|e| ControlError::Io(e.to_string()))? {
            let entry = entry.map_err(|e| ControlError::Io(e.to_string()))?;
            let meta = entry.path().join("account.toml");
            if meta.is_file() {
                let raw = fs::read_to_string(&meta).map_err(|e| ControlError::Io(e.to_string()))?;
                let rec: AccountRecord =
                    toml::from_str(&raw).map_err(|e| ControlError::Config(e.to_string()))?;
                out.push(rec);
            }
        }
        out.sort_by(|a, b| a.key.cmp(&b.key));
        Ok(out)
    }

    pub fn credentials(&self) -> Result<TdlibCredentials, ControlError> {
        let id = self.config.telegram.api_id.unwrap_or(0);
        let hash = self.config.telegram.api_hash.clone().unwrap_or_default();
        if id == 0 || hash.is_empty() {
            return Err(ControlError::Config(
                "Telegram application credentials missing. Set [telegram] api_id / api_hash in ~/.gram/config.toml or TELEGRAM_API_ID / TELEGRAM_API_HASH".into(),
            ));
        }
        Ok(TdlibCredentials {
            api_id: id,
            api_hash: hash,
            use_test_dc: self.config.telegram.use_test_dc,
        })
    }

    fn open_account(&self, rec: &AccountRecord) -> Result<Account, ControlError> {
        let uuid = Uuid::parse_str(&rec.account_id)
            .map_err(|e| ControlError::Config(format!("invalid account_id: {e}")))?;
        let session = self.paths.account_session(&rec.key);
        Ok(Account::with_id(
            TelegramAccountId(uuid),
            session,
            self.credentials()?,
        ))
    }

    pub fn ensure_record(&self, phone: &str) -> Result<AccountRecord, ControlError> {
        let phone = normalize_phone(phone);
        let key = sanitize_key(&phone);
        let meta = self.paths.account_meta(&key);
        if meta.is_file() {
            let raw = fs::read_to_string(&meta).map_err(|e| ControlError::Io(e.to_string()))?;
            return toml::from_str(&raw).map_err(|e| ControlError::Config(e.to_string()));
        }
        let dir = self.paths.account(&key);
        fs::create_dir_all(self.paths.account_session(&key))
            .map_err(|e| ControlError::Io(e.to_string()))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&dir, fs::Permissions::from_mode(0o700));
        }
        let rec = AccountRecord {
            key: key.clone(),
            phone,
            account_id: Uuid::now_v7().to_string(),
            display_name: None,
            username: None,
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        self.write_meta(&rec)?;
        Ok(rec)
    }

    pub fn load_record(&self, key: &str) -> Result<AccountRecord, ControlError> {
        let key = sanitize_key(key);
        let path = self.paths.account_meta(&key);
        let raw = fs::read_to_string(&path)
            .map_err(|_| ControlError::NotFound(format!("account `{key}` not found")))?;
        toml::from_str(&raw).map_err(|e| ControlError::Config(e.to_string()))
    }

    fn write_meta(&self, rec: &AccountRecord) -> Result<(), ControlError> {
        let path = self.paths.account_meta(&rec.key);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| ControlError::Io(e.to_string()))?;
        }
        let raw = toml::to_string_pretty(rec).map_err(|e| ControlError::Config(e.to_string()))?;
        fs::write(&path, raw).map_err(|e| ControlError::Io(e.to_string()))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o600));
        }
        Ok(())
    }

    fn phase_from_status(status: TelegramAccountStatus) -> AuthPhase {
        match status {
            TelegramAccountStatus::WaitPhoneNumber
            | TelegramAccountStatus::Created
            | TelegramAccountStatus::Initializing => AuthPhase::PhoneRequired,
            TelegramAccountStatus::WaitCode => AuthPhase::CodeRequired,
            TelegramAccountStatus::WaitPassword => AuthPhase::PasswordRequired,
            TelegramAccountStatus::WaitRegistration => AuthPhase::RegistrationRequired,
            TelegramAccountStatus::Ready => AuthPhase::Authorized,
            _ => AuthPhase::Error,
        }
    }

    async fn wait_phase(
        account: &Account,
        timeout: Duration,
    ) -> Result<(TelegramAccountStatus, AuthPhase), ControlError> {
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            let status = account.auth_state().await;
            let phase = Self::phase_from_status(status);
            if !matches!(
                status,
                TelegramAccountStatus::Initializing | TelegramAccountStatus::Created
            ) {
                return Ok((status, phase));
            }
            if tokio::time::Instant::now() >= deadline {
                return Err(ControlError::Unavailable(
                    "timed out waiting for TDLib auth state".into(),
                ));
            }
            tokio::time::sleep(Duration::from_millis(150)).await;
        }
    }

    /// Interactive / stepwise login using library Account.
    pub async fn login(
        &self,
        req: LoginRequest,
        mut prompts: LoginPrompts<'_>,
    ) -> Result<LoginResult, ControlError> {
        let rec = self.ensure_record(&req.phone)?;
        let account = self.open_account(&rec)?;
        account.start().await?;
        let (_, mut phase) = Self::wait_phase(&account, Duration::from_secs(60)).await?;

        if phase == AuthPhase::Authorized {
            return self.finish_authorized(&rec, &account).await;
        }

        if phase == AuthPhase::PhoneRequired {
            account.submit_phone(rec.phone.clone()).await?;
            let (_, p) = Self::wait_phase(&account, Duration::from_secs(60)).await?;
            phase = p;
        }

        if phase == AuthPhase::CodeRequired {
            let code = if let Some(c) = req.code.clone() {
                c
            } else if let LoginPrompts::Interactive { code, .. } = &mut prompts {
                code()?
            } else if let Ok(c) = std::env::var("GRAM_AUTH_CODE") {
                c
            } else {
                return Ok(LoginResult {
                    key: rec.key.clone(),
                    phase: AuthPhase::CodeRequired,
                    message: "authentication code required".into(),
                    view: None,
                });
            };
            account
                .submit_code(code)
                .await
                .map_err(|e| ControlError::Auth(format!("invalid authentication code ({e})")))?;
            let (_, p) = Self::wait_phase(&account, Duration::from_secs(60)).await?;
            phase = p;
        }

        if phase == AuthPhase::PasswordRequired {
            let password = if let Some(p) = req.password.clone() {
                p
            } else if let LoginPrompts::Interactive { password, .. } = &mut prompts {
                password()?
            } else if let Ok(p) = std::env::var("GRAM_2FA_PASSWORD") {
                p
            } else {
                return Ok(LoginResult {
                    key: rec.key.clone(),
                    phase: AuthPhase::PasswordRequired,
                    message: "2FA password required".into(),
                    view: None,
                });
            };
            account
                .submit_password(password)
                .await
                .map_err(|e| ControlError::Auth(format!("2FA failed ({e})")))?;
            let (_, p) = Self::wait_phase(&account, Duration::from_secs(60)).await?;
            phase = p;
        }

        if phase == AuthPhase::RegistrationRequired {
            let first = if let Some(n) = req.first_name.clone() {
                n
            } else if let LoginPrompts::Interactive { first_name, .. } = &mut prompts {
                first_name()?
            } else {
                return Ok(LoginResult {
                    key: rec.key.clone(),
                    phase: AuthPhase::RegistrationRequired,
                    message: "registration first name required".into(),
                    view: None,
                });
            };
            account.register_user(first, None).await?;
            let (_, p) = Self::wait_phase(&account, Duration::from_secs(60)).await?;
            phase = p;
        }

        if phase == AuthPhase::Authorized {
            return self.finish_authorized(&rec, &account).await;
        }

        let _ = account.shutdown().await;
        Err(ControlError::Auth(format!(
            "authentication incomplete (phase={phase:?})"
        )))
    }

    async fn finish_authorized(
        &self,
        rec: &AccountRecord,
        account: &Account,
    ) -> Result<LoginResult, ControlError> {
        let mut rec = rec.clone();
        if let Ok(profile) = account.profile().await {
            rec.display_name = profile.display_name;
            rec.username = profile.username;
        }
        self.write_meta(&rec)?;
        let view = AccountView {
            key: rec.key.clone(),
            phone: rec.phone.clone(),
            status: "authorized".into(),
            display_name: rec.display_name.clone(),
            username: rec.username.clone(),
            data_dir: self.paths.account(&rec.key).display().to_string(),
        };
        let _ = account.shutdown().await;
        Ok(LoginResult {
            key: rec.key,
            phase: AuthPhase::Authorized,
            message: "account authorized".into(),
            view: Some(view),
        })
    }

    fn live_get(&self, key: &str) -> Option<Arc<Account>> {
        self.live.lock().unwrap().get(key).cloned()
    }

    fn live_insert(&self, key: String, account: Arc<Account>) {
        self.live.lock().unwrap().insert(key, account);
    }

    fn live_remove(&self, key: &str) -> Option<Arc<Account>> {
        self.live.lock().unwrap().remove(key)
    }

    fn live_contains(&self, key: &str) -> bool {
        self.live.lock().unwrap().contains_key(key)
    }

    pub async fn status(&self, key: &str) -> Result<AccountView, ControlError> {
        let rec = self.load_record(key)?;
        if let Some(live) = self.live_get(&rec.key) {
            let status = live.auth_state().await;
            return Ok(AccountView {
                key: rec.key,
                phone: rec.phone,
                status: status.as_str().into(),
                display_name: rec.display_name,
                username: rec.username,
                data_dir: self.paths.account(key).display().to_string(),
            });
        }
        let account = self.open_account(&rec)?;
        match account.start().await {
            Ok(()) => {
                let status = account.auth_state().await;
                let _ = account.shutdown().await;
                Ok(AccountView {
                    key: rec.key,
                    phone: rec.phone,
                    status: status.as_str().into(),
                    display_name: rec.display_name,
                    username: rec.username,
                    data_dir: self.paths.account(key).display().to_string(),
                })
            }
            Err(e) => Ok(AccountView {
                key: rec.key,
                phone: rec.phone,
                status: format!("error: {e}"),
                display_name: rec.display_name,
                username: rec.username,
                data_dir: self.paths.account(key).display().to_string(),
            }),
        }
    }

    pub async fn start(&self, key: &str) -> Result<AccountView, ControlError> {
        let rec = self.load_record(key)?;
        let account = Arc::new(self.open_account(&rec)?);
        account.start().await?;
        self.live_insert(rec.key.clone(), account.clone());
        let status = account.auth_state().await;
        Ok(AccountView {
            key: rec.key,
            phone: rec.phone,
            status: status.as_str().into(),
            display_name: rec.display_name,
            username: rec.username,
            data_dir: self.paths.account(key).display().to_string(),
        })
    }

    pub async fn stop(&self, key: &str) -> Result<(), ControlError> {
        let key = sanitize_key(key);
        if let Some(acc) = self.live_remove(&key) {
            let _ = acc.shutdown().await;
        }
        Ok(())
    }

    pub async fn logout(&self, key: &str) -> Result<(), ControlError> {
        let rec = self.load_record(key)?;
        let _ = self.stop(&rec.key).await;
        let account = self.open_account(&rec)?;
        let _ = account.start().await;
        account.logout().await?;
        Ok(())
    }

    pub async fn remove(&self, key: &str) -> Result<(), ControlError> {
        let rec = self.load_record(key)?;
        let _ = self.stop(&rec.key).await;
        let dir = self.paths.account(&rec.key);
        fs::remove_dir_all(&dir).map_err(|e| ControlError::Io(e.to_string()))?;
        Ok(())
    }

    pub async fn send(&self, key: &str, chat: &str, text: &str) -> Result<i64, ControlError> {
        let rec = self.load_record(key)?;
        let account = if let Some(a) = self.live_get(&rec.key) {
            a
        } else {
            let a = Arc::new(self.open_account(&rec)?);
            a.start().await?;
            let status = a.auth_state().await;
            if status != TelegramAccountStatus::Ready {
                let _ = a.shutdown().await;
                return Err(ControlError::Auth(format!(
                    "account not ready ({})",
                    status.as_str()
                )));
            }
            a
        };

        let chat_id = if let Ok(id) = chat.parse::<i64>() {
            id
        } else {
            let username = chat.trim().trim_start_matches('@');
            let found = account.search_chats(username, Some(5)).await?;
            found
                .chats
                .into_iter()
                .find(|c| {
                    c.username
                        .as_ref()
                        .map(|u| u.eq_ignore_ascii_case(username))
                        .unwrap_or(false)
                })
                .map(|c| c.id.0)
                .ok_or_else(|| ControlError::NotFound(format!("chat `{chat}` not found")))?
        };

        let msg_id = account.send_message(chat_id, text).await?;
        if !self.live_contains(&rec.key) {
            let _ = account.shutdown().await;
        }
        Ok(msg_id.0)
    }

    pub fn info(&self, key: &str) -> Result<AccountRecord, ControlError> {
        self.load_record(key)
    }
}
