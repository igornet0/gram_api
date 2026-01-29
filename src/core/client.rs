//! HTTP client for Telegram Bot API.

use crate::core::error::TelegramError;
use crate::core::models::Update;
use serde::de::DeserializeOwned;
use serde::Serialize;

/// Raw API response wrapper. Telegram returns `{ ok: true, result: T }` or `{ ok: false, description, error_code }`.
#[derive(Debug, serde::Deserialize)]
pub struct ApiResponse<T> {
    pub ok: bool,
    pub result: Option<T>,
    pub description: Option<String>,
    pub error_code: Option<i32>,
}

/// Low-level HTTP client for Telegram Bot API. No bot or handler logic.
#[derive(Clone)]
pub struct ApiClient {
    #[allow(dead_code)]
    token: String,
    client: reqwest::Client,
    base_url: String,
}

impl ApiClient {
    /// Create a new API client with the given bot token.
    pub fn new(token: impl Into<String>) -> Self {
        let token = token.into();
        let base_url = format!("https://api.telegram.org/bot{}/", token);
        Self {
            token: token.clone(),
            client: reqwest::Client::new(),
            base_url,
        }
    }

    /// Create with a custom reqwest client (e.g. for timeouts or proxy).
    pub fn with_client(token: impl Into<String>, client: reqwest::Client) -> Self {
        let token = token.into();
        let base_url = format!("https://api.telegram.org/bot{}/", token);
        Self {
            token,
            client,
            base_url,
        }
    }

    /// GET request to method, parse JSON into T.
    pub async fn get<T>(&self, method: &str) -> Result<T, TelegramError>
    where
        T: DeserializeOwned,
    {
        let url = format!("{}{}", self.base_url, method);
        let res = self.client.get(&url).send().await?;
        self.parse_response(res).await
    }

    /// POST request with JSON body, parse response into T.
    pub async fn post<T, B>(&self, method: &str, body: &B) -> Result<T, TelegramError>
    where
        T: DeserializeOwned,
        B: Serialize + ?Sized,
    {
        let url = format!("{}{}", self.base_url, method);
        let res = self.client.post(&url).json(body).send().await?;
        self.parse_response(res).await
    }

    async fn parse_response<T>(&self, res: reqwest::Response) -> Result<T, TelegramError>
    where
        T: DeserializeOwned,
    {
        let bytes = res.bytes().await?;
        let api: ApiResponse<T> = serde_json::from_slice(&bytes)?;
        if api.ok {
            api.result
                .ok_or_else(|| TelegramError::Api {
                    description: "ok=true but no result".to_string(),
                    error_code: None,
                })
        } else {
            Err(TelegramError::Api {
                description: api.description.unwrap_or_else(|| "Unknown error".to_string()),
                error_code: api.error_code,
            })
        }
    }

    /// Get updates (long polling). Used by the runtime.
    pub async fn get_updates(
        &self,
        offset: i64,
        timeout: u64,
        limit: Option<u32>,
    ) -> Result<Vec<Update>, TelegramError> {
        #[derive(Serialize)]
        struct GetUpdates {
            offset: i64,
            timeout: u64,
            #[serde(skip_serializing_if = "Option::is_none")]
            limit: Option<u32>,
        }
        let body = GetUpdates {
            offset,
            timeout,
            limit,
        };
        self.post("getUpdates", &body).await
    }
}
