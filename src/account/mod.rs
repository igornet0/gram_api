//! User-account (TDLib) API powered by [`tglib`].
//!
//! Available only with feature **`accounts`** (links `tdjson`).
//! Bots use [`crate::Bot`] (HTTP Bot API) without this feature.

#[allow(clippy::module_inception)]
mod account;
mod manager;

pub use account::Account;
pub use manager::AccountManager;

pub use tglib::{
    DeleteMessagesRequest, DownloadFileRequest, DownloadedFile, EditMessageRequest,
    ForwardMessagesRequest, GetMessagesRequest, JoinPublicChatRequest, ListChatsRequest,
    ListChatsResponse, RegisterUserRequest, SearchChatsRequest, SendDocumentRequest,
    SendMessageRequest, SendPhotoRequest, TdlibAdapter, TdlibClientFactory, TdlibCredentials,
    TelegramAccountId, TelegramAccountProfile, TelegramAccountStatus, TelegramChat, TelegramChatId,
    TelegramClient, TelegramClientConfig, TelegramClientFactory, TelegramClientUpdate,
    TelegramConfigInput, TelegramConfigProvider, TelegramConfigResolve,
    TelegramError as AccountError, TelegramMessage, TelegramMessageId, TelegramUserId,
    SECRET_API_HASH, SECRET_API_ID,
};

/// Re-export of the underlying `tglib` crate for advanced use.
pub mod raw {
    pub use tglib::*;
}
