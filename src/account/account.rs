//! High-level user-account handle (TDLib).

use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::sync::broadcast;

use super::{
    AccountError, DeleteMessagesRequest, DownloadFileRequest, DownloadedFile, EditMessageRequest,
    ForwardMessagesRequest, GetMessagesRequest, JoinPublicChatRequest, ListChatsRequest,
    ListChatsResponse, RegisterUserRequest, SearchChatsRequest, SendDocumentRequest,
    SendMessageRequest, SendPhotoRequest, TdlibAdapter, TdlibCredentials, TelegramAccountId,
    TelegramAccountProfile, TelegramAccountStatus, TelegramChat, TelegramChatId, TelegramClient,
    TelegramClientUpdate, TelegramMessage, TelegramMessageId,
};

/// User Telegram account via TDLib.
///
/// Complements [`crate::Bot`] (Bot API). Requires feature `accounts`.
pub struct Account {
    account_id: TelegramAccountId,
    session_root: PathBuf,
    client: Arc<dyn TelegramClient>,
}

impl Account {
    /// Create a TDLib-backed account (new id).
    pub fn new(session_root: impl Into<PathBuf>, credentials: TdlibCredentials) -> Self {
        Self::with_id(TelegramAccountId::new(), session_root, credentials)
    }

    /// Create a TDLib-backed account with a fixed id (session restore).
    pub fn with_id(
        account_id: TelegramAccountId,
        session_root: impl Into<PathBuf>,
        credentials: TdlibCredentials,
    ) -> Self {
        let session_root = session_root.into();
        let _ = std::fs::create_dir_all(&session_root);
        let client: Arc<dyn TelegramClient> = Arc::new(TdlibAdapter::new(
            account_id,
            session_root.clone(),
            credentials,
        ));
        Self {
            account_id,
            session_root,
            client,
        }
    }

    /// Wrap any [`TelegramClient`] (e.g. custom factory).
    pub fn from_client(
        account_id: TelegramAccountId,
        session_root: impl Into<PathBuf>,
        client: Arc<dyn TelegramClient>,
    ) -> Self {
        Self {
            account_id,
            session_root: session_root.into(),
            client,
        }
    }

    pub fn id(&self) -> TelegramAccountId {
        self.account_id
    }

    pub fn session_root(&self) -> &Path {
        &self.session_root
    }

    pub fn client(&self) -> &Arc<dyn TelegramClient> {
        &self.client
    }

    pub async fn start(&self) -> Result<(), AccountError> {
        self.client.start().await
    }

    pub async fn shutdown(&self) -> Result<(), AccountError> {
        self.client.shutdown().await
    }

    pub async fn auth_state(&self) -> TelegramAccountStatus {
        self.client.auth_state().await
    }

    pub async fn submit_phone(&self, phone: impl Into<String>) -> Result<(), AccountError> {
        self.client.submit_phone(phone.into()).await
    }

    pub async fn submit_code(&self, code: impl Into<String>) -> Result<(), AccountError> {
        self.client.submit_code(code.into()).await
    }

    pub async fn submit_password(&self, password: impl Into<String>) -> Result<(), AccountError> {
        self.client.submit_password(password.into()).await
    }

    pub async fn register_user(
        &self,
        first_name: impl Into<String>,
        last_name: Option<String>,
    ) -> Result<(), AccountError> {
        self.client
            .register_user(RegisterUserRequest {
                first_name: first_name.into(),
                last_name,
            })
            .await
    }

    pub async fn logout(&self) -> Result<(), AccountError> {
        self.client.logout().await
    }

    pub async fn profile(&self) -> Result<TelegramAccountProfile, AccountError> {
        self.client.get_account_info().await
    }

    pub async fn list_chats(&self, limit: Option<u32>) -> Result<ListChatsResponse, AccountError> {
        self.client.list_chats(ListChatsRequest { limit }).await
    }

    pub async fn search_chats(
        &self,
        query: impl Into<String>,
        limit: Option<u32>,
    ) -> Result<ListChatsResponse, AccountError> {
        self.client
            .search_chats(SearchChatsRequest {
                query: query.into(),
                limit,
            })
            .await
    }

    pub async fn get_chat(&self, chat_id: i64) -> Result<TelegramChat, AccountError> {
        self.client.get_chat(TelegramChatId(chat_id)).await
    }

    pub async fn get_messages(
        &self,
        chat_id: i64,
        limit: Option<u32>,
    ) -> Result<Vec<TelegramMessage>, AccountError> {
        self.client
            .get_messages(GetMessagesRequest {
                chat_id: TelegramChatId(chat_id),
                limit,
            })
            .await
    }

    pub async fn send_message(
        &self,
        chat_id: i64,
        text: impl Into<String>,
    ) -> Result<TelegramMessageId, AccountError> {
        self.client
            .send_message(SendMessageRequest {
                chat_id: TelegramChatId(chat_id),
                text: text.into(),
                parse_mode: tglib::ParseMode::Plain,
            })
            .await
    }

    pub async fn send_message_html(
        &self,
        chat_id: i64,
        text: impl Into<String>,
    ) -> Result<TelegramMessageId, AccountError> {
        self.client
            .send_message(SendMessageRequest {
                chat_id: TelegramChatId(chat_id),
                text: text.into(),
                parse_mode: tglib::ParseMode::Html,
            })
            .await
    }

    pub async fn forward_messages(
        &self,
        from_chat_id: i64,
        to_chat_id: i64,
        message_ids: impl IntoIterator<Item = i64>,
    ) -> Result<(), AccountError> {
        self.client
            .forward_messages(ForwardMessagesRequest {
                from_chat_id: TelegramChatId(from_chat_id),
                to_chat_id: TelegramChatId(to_chat_id),
                message_ids: message_ids.into_iter().map(TelegramMessageId).collect(),
            })
            .await
    }

    pub async fn edit_message(
        &self,
        chat_id: i64,
        message_id: i64,
        text: impl Into<String>,
    ) -> Result<(), AccountError> {
        self.client
            .edit_message(EditMessageRequest {
                chat_id: TelegramChatId(chat_id),
                message_id: TelegramMessageId(message_id),
                text: text.into(),
                parse_mode: tglib::ParseMode::Plain,
            })
            .await
    }

    pub async fn delete_messages(
        &self,
        chat_id: i64,
        message_ids: impl IntoIterator<Item = i64>,
    ) -> Result<(), AccountError> {
        self.client
            .delete_messages(DeleteMessagesRequest {
                chat_id: TelegramChatId(chat_id),
                message_ids: message_ids.into_iter().map(TelegramMessageId).collect(),
            })
            .await
    }

    pub async fn send_photo(
        &self,
        chat_id: i64,
        path: impl Into<String>,
        caption: Option<String>,
    ) -> Result<TelegramMessageId, AccountError> {
        self.client
            .send_photo(SendPhotoRequest {
                chat_id: TelegramChatId(chat_id),
                path: path.into(),
                caption,
                parse_mode: tglib::ParseMode::Plain,
            })
            .await
    }

    pub async fn send_document(
        &self,
        chat_id: i64,
        path: impl Into<String>,
        caption: Option<String>,
    ) -> Result<TelegramMessageId, AccountError> {
        self.client
            .send_document(SendDocumentRequest {
                chat_id: TelegramChatId(chat_id),
                path: path.into(),
                caption,
                parse_mode: tglib::ParseMode::Plain,
            })
            .await
    }

    pub async fn download_file(
        &self,
        file_id: i32,
        priority: Option<i32>,
    ) -> Result<DownloadedFile, AccountError> {
        self.client
            .download_file(DownloadFileRequest { file_id, priority })
            .await
    }

    pub async fn join_public_chat(
        &self,
        username: impl Into<String>,
    ) -> Result<TelegramChat, AccountError> {
        self.client
            .join_public_chat(JoinPublicChatRequest {
                username: username.into(),
            })
            .await
    }

    pub async fn leave_chat(&self, chat_id: i64) -> Result<(), AccountError> {
        self.client.leave_chat(TelegramChatId(chat_id)).await
    }

    /// Subscribe to user-account updates (messages, auth, chats, …).
    pub fn subscribe(&self) -> broadcast::Receiver<TelegramClientUpdate> {
        self.client.subscribe()
    }
}
