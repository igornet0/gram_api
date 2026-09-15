//! Control-plane errors and CLI exit codes.

use std::fmt;

#[derive(Debug)]
pub enum ControlError {
    Io(String),
    Config(String),
    NotFound(String),
    Auth(String),
    Invalid(String),
    Unavailable(String),
    #[cfg(feature = "accounts")]
    Account(crate::account::AccountError),
    Bot(crate::core::TelegramError),
}

impl fmt::Display for ControlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(m)
            | Self::Config(m)
            | Self::NotFound(m)
            | Self::Auth(m)
            | Self::Invalid(m)
            | Self::Unavailable(m) => write!(f, "{m}"),
            #[cfg(feature = "accounts")]
            Self::Account(e) => write!(f, "{e}"),
            Self::Bot(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for ControlError {}

#[cfg(feature = "accounts")]
impl From<crate::account::AccountError> for ControlError {
    fn from(value: crate::account::AccountError) -> Self {
        Self::Account(value)
    }
}

impl From<crate::core::TelegramError> for ControlError {
    fn from(value: crate::core::TelegramError) -> Self {
        Self::Bot(value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum ExitCode {
    Success = 0,
    General = 1,
    Usage = 2,
    AuthFailed = 3,
    NotFound = 4,
    Unavailable = 5,
}

impl ExitCode {
    pub fn from_error(err: &ControlError) -> Self {
        match err {
            ControlError::Invalid(_) | ControlError::Config(_) => Self::Usage,
            ControlError::Auth(_) => Self::AuthFailed,
            ControlError::NotFound(_) => Self::NotFound,
            ControlError::Unavailable(_) => Self::Unavailable,
            #[cfg(feature = "accounts")]
            ControlError::Account(e) => {
                let s = e.to_string().to_ascii_lowercase();
                if s.contains("auth") || s.contains("code") || s.contains("password") {
                    Self::AuthFailed
                } else if s.contains("not found") {
                    Self::NotFound
                } else {
                    Self::General
                }
            }
            _ => Self::General,
        }
    }
}
