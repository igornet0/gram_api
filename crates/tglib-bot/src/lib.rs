//! Telegram Bot API HTTP client.
//!
//! User/TDLib accounts live in `tglib-client` / `tglib-tdlib`. This crate talks to
//! `https://api.telegram.org/bot<token>/…` (or a custom `api_base`).

use serde_json::{Value, json};
use thiserror::Error;
use tglib_client::{prepare_telegram_text, ParseMode};

const DEFAULT_API_BASE: &str = "https://api.telegram.org";

/// Bot API errors. Do not put the bot token in `message`.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum BotError {
    #[error("authentication failed: {0}")]
    Authentication(String),
    #[error("invalid request: {0}")]
    InvalidRequest(String),
    #[error("rate limited: {0}")]
    RateLimit(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("forbidden: {0}")]
    Forbidden(String),
    #[error("upstream error: {0}")]
    Upstream(String),
}

impl BotError {
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::RateLimit(_) | Self::Upstream(_))
    }

    pub fn kind(&self) -> &'static str {
        match self {
            Self::Authentication(_) => "authentication",
            Self::InvalidRequest(_) => "invalid_request",
            Self::RateLimit(_) => "rate_limit",
            Self::NotFound(_) => "not_found",
            Self::Forbidden(_) => "forbidden",
            Self::Upstream(_) => "upstream",
        }
    }
}

/// HTTP client for Telegram Bot API methods used by workflow nodes.
pub struct BotClient {
    pub api_base: String,
    pub bot_token: String,
    client: reqwest::Client,
}

impl BotClient {
    pub fn new(token: impl Into<String>) -> Result<Self, BotError> {
        let bot_token = token.into();
        if bot_token.trim().is_empty() {
            return Err(BotError::Authentication("empty bot_token".into()));
        }
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| BotError::Upstream(e.to_string()))?;
        Ok(Self {
            api_base: DEFAULT_API_BASE.to_string(),
            bot_token,
            client,
        })
    }

    pub fn with_api_base(mut self, base: impl Into<String>) -> Self {
        let base = base.into().trim_end_matches('/').to_string();
        if !base.is_empty() {
            self.api_base = base;
        }
        self
    }

    fn method_url(&self, method: &str) -> String {
        format!("{}/bot{}/{}", self.api_base, self.bot_token, method)
    }

    pub async fn send_message(
        &self,
        chat_id: &str,
        text: &str,
        parse_mode: Option<&str>,
    ) -> Result<Value, BotError> {
        let mode = ParseMode::parse(parse_mode);
        let (body_text, bot_parse_mode) = prepare_telegram_text(text, mode);
        let mut body = json!({
            "chat_id": chat_id,
            "text": body_text,
        });
        if let Some(pm) = bot_parse_mode {
            body["parse_mode"] = Value::String(pm.into());
        }
        self.post_json("sendMessage", &body).await
    }

    pub async fn send_photo(
        &self,
        chat_id: &str,
        photo: &str,
        caption: Option<&str>,
    ) -> Result<Value, BotError> {
        let mut body = json!({
            "chat_id": chat_id,
            "photo": photo,
        });
        if let Some(c) = caption {
            body["caption"] = Value::String(c.to_string());
        }
        self.post_json("sendPhoto", &body).await
    }

    pub async fn send_document(
        &self,
        chat_id: &str,
        document: &str,
        caption: Option<&str>,
    ) -> Result<Value, BotError> {
        let mut body = json!({
            "chat_id": chat_id,
            "document": document,
        });
        if let Some(c) = caption {
            body["caption"] = Value::String(c.to_string());
        }
        self.post_json("sendDocument", &body).await
    }

    pub async fn get_me(&self) -> Result<Value, BotError> {
        self.post_json("getMe", &json!({})).await
    }

    async fn post_json(&self, method: &str, body: &Value) -> Result<Value, BotError> {
        let url = self.method_url(method);
        let response = self
            .client
            .post(&url)
            .json(body)
            .send()
            .await
            .map_err(|e| BotError::Upstream(e.to_string()))?;

        let status = response.status();
        let payload: Value = response
            .json()
            .await
            .map_err(|e| BotError::Upstream(format!("invalid JSON response: {e}")))?;

        if payload.get("ok").and_then(Value::as_bool) == Some(true) {
            return Ok(payload.get("result").cloned().unwrap_or(payload));
        }

        let description = payload
            .get("description")
            .and_then(Value::as_str)
            .unwrap_or("telegram API error")
            .to_string();
        let error_code = payload
            .get("error_code")
            .and_then(Value::as_u64)
            .unwrap_or(0);

        Err(map_bot_error(status.as_u16(), error_code, description))
    }
}

pub fn map_bot_error(http_status: u16, error_code: u64, description: String) -> BotError {
    let code = if error_code != 0 {
        error_code
    } else {
        http_status as u64
    };
    match code {
        401 | 403 if description.to_lowercase().contains("unauthorized") => {
            BotError::Authentication(description)
        }
        401 => BotError::Authentication(description),
        403 => BotError::Forbidden(description),
        404 => BotError::NotFound(description),
        429 => BotError::RateLimit(description),
        400 => BotError::InvalidRequest(description),
        _ if http_status >= 500 || code >= 500 => BotError::Upstream(description),
        _ => BotError::Upstream(description),
    }
}

/// Extract bot token for internal use only — never log the return value.
pub fn extract_bot_token(credentials: &Value) -> Option<&str> {
    credentials
        .get("bot_token")
        .or_else(|| credentials.get("token"))
        .and_then(Value::as_str)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    async fn serve_once(listener: TcpListener, status: u16, body: &str) {
        let body = body.to_string();
        let (mut stream, _) = listener.accept().await.unwrap();
        let mut buf = vec![0u8; 8192];
        let _ = stream.read(&mut buf).await;
        let reason = if status == 200 { "OK" } else { "ERR" };
        let resp = format!(
            "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        stream.write_all(resp.as_bytes()).await.unwrap();
    }

    #[test]
    fn rejects_empty_token() {
        assert!(matches!(
            BotClient::new(""),
            Err(BotError::Authentication(_))
        ));
        assert!(matches!(
            BotClient::new("   "),
            Err(BotError::Authentication(_))
        ));
    }

    #[test]
    fn maps_http_errors() {
        assert!(matches!(
            map_bot_error(401, 401, "Unauthorized".into()),
            BotError::Authentication(_)
        ));
        assert!(matches!(
            map_bot_error(400, 400, "chat not found".into()),
            BotError::InvalidRequest(_)
        ));
        assert!(matches!(
            map_bot_error(429, 429, "retry".into()),
            BotError::RateLimit(_)
        ));
        assert!(BotError::RateLimit("x".into()).is_retryable());
        assert_eq!(BotError::NotFound("x".into()).kind(), "not_found");
    }

    #[test]
    fn extracts_token_fields() {
        assert_eq!(
            extract_bot_token(&json!({ "bot_token": "abc" })),
            Some("abc")
        );
        assert_eq!(extract_bot_token(&json!({ "token": "xyz" })), Some("xyz"));
        assert_eq!(extract_bot_token(&json!({})), None);
    }

    #[tokio::test]
    async fn get_me_ok() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let body = r#"{"ok":true,"result":{"id":1,"is_bot":true,"username":"demo_bot"}}"#;
        let server = tokio::spawn(serve_once(listener, 200, body));

        let client = BotClient::new("123:token")
            .unwrap()
            .with_api_base(format!("http://{addr}"));
        let me = client.get_me().await.unwrap();
        assert_eq!(me["username"], "demo_bot");
        server.await.unwrap();
    }

    #[tokio::test]
    async fn send_message_applies_markdown() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let body = r#"{"ok":true,"result":{"message_id":7}}"#;
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut buf = vec![0u8; 16384];
            let n = stream.read(&mut buf).await.unwrap();
            let req = String::from_utf8_lossy(&buf[..n]).to_string();
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            stream.write_all(resp.as_bytes()).await.unwrap();
            req
        });

        let client = BotClient::new("123:token")
            .unwrap()
            .with_api_base(format!("http://{addr}"));
        let result = client
            .send_message("42", "**hi**", Some("markdown"))
            .await
            .unwrap();
        assert_eq!(result["message_id"], 7);
        let req = server.await.unwrap();
        assert!(req.contains("/bot123:token/sendMessage"));
        assert!(req.contains("<b>hi</b>"));
        assert!(req.contains("HTML"));
    }

    #[tokio::test]
    async fn unauthorized_get_me() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let body = r#"{"ok":false,"error_code":401,"description":"Unauthorized"}"#;
        let server = tokio::spawn(serve_once(listener, 401, body));

        let client = BotClient::new("bad")
            .unwrap()
            .with_api_base(format!("http://{addr}"));
        let err = client.get_me().await.unwrap_err();
        assert!(matches!(err, BotError::Authentication(_)));
        server.await.unwrap();
    }
}
