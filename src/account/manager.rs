//! Multi-account registry for TDLib user sessions.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use super::{Account, AccountError, TdlibCredentials, TelegramAccountId, TelegramAccountStatus};

/// In-memory multi-account manager (TDLib sessions on disk).
pub struct AccountManager {
    data_dir: PathBuf,
    accounts: Mutex<HashMap<TelegramAccountId, Arc<Account>>>,
}

impl AccountManager {
    pub fn new(data_dir: impl Into<PathBuf>) -> Self {
        let data_dir = data_dir.into();
        let _ = std::fs::create_dir_all(&data_dir);
        Self {
            data_dir,
            accounts: Mutex::new(HashMap::new()),
        }
    }

    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    fn session_dir(&self, id: TelegramAccountId) -> PathBuf {
        self.data_dir.join("accounts").join(id.to_string())
    }

    /// Create and register a TDLib account.
    pub fn create(&self, credentials: TdlibCredentials) -> Arc<Account> {
        let id = TelegramAccountId::new();
        let account = Arc::new(Account::with_id(id, self.session_dir(id), credentials));
        self.accounts
            .lock()
            .expect("account manager lock")
            .insert(id, account.clone());
        account
    }

    /// Register an existing account handle.
    pub fn insert(&self, account: Arc<Account>) {
        let id = account.id();
        self.accounts
            .lock()
            .expect("account manager lock")
            .insert(id, account);
    }

    pub fn get(&self, id: TelegramAccountId) -> Option<Arc<Account>> {
        self.accounts
            .lock()
            .expect("account manager lock")
            .get(&id)
            .cloned()
    }

    pub fn list_ids(&self) -> Vec<TelegramAccountId> {
        self.accounts
            .lock()
            .expect("account manager lock")
            .keys()
            .copied()
            .collect()
    }

    pub async fn start(&self, id: TelegramAccountId) -> Result<(), AccountError> {
        let account = self.get(id).ok_or(AccountError::AccountNotFound)?;
        account.start().await
    }

    pub async fn status(
        &self,
        id: TelegramAccountId,
    ) -> Result<TelegramAccountStatus, AccountError> {
        let account = self.get(id).ok_or(AccountError::AccountNotFound)?;
        Ok(account.auth_state().await)
    }

    pub async fn remove(&self, id: TelegramAccountId) -> Result<(), AccountError> {
        let account = self
            .accounts
            .lock()
            .expect("account manager lock")
            .remove(&id);
        if let Some(account) = account {
            let _ = account.shutdown().await;
        }
        Ok(())
    }

    pub async fn shutdown_all(&self) {
        let accounts: Vec<_> = self
            .accounts
            .lock()
            .expect("account manager lock")
            .values()
            .cloned()
            .collect();
        for account in accounts {
            let _ = account.shutdown().await;
        }
    }
}
