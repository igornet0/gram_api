//! Webhook server: receive updates via HTTP POST.

#[cfg(feature = "webhook")]
use axum::{extract::State, routing::post, Json, Router};
#[cfg(feature = "webhook")]
use std::sync::Arc;

#[cfg(feature = "webhook")]
use crate::core::Update;
#[cfg(feature = "webhook")]
use crate::dispatcher::Dispatcher;
#[cfg(feature = "webhook")]
use crate::Bot;

/// State held by the webhook server.
#[cfg(feature = "webhook")]
#[derive(Clone)]
pub struct WebhookState {
    pub bot: Arc<Bot>,
    pub dispatcher: Arc<Dispatcher>,
}

/// Create the webhook router. Mount at e.g. `/telegram/webhook`.
/// The handler parses the body as `Update`, runs the dispatcher, and returns 200.
#[cfg(feature = "webhook")]
pub fn webhook_router(state: WebhookState) -> Router {
    Router::new()
        .route("/telegram/webhook", post(webhook_handler))
        .with_state(state)
}

#[cfg(feature = "webhook")]
async fn webhook_handler(
    State(state): State<WebhookState>,
    Json(update): Json<Update>,
) -> (axum::http::StatusCode, &'static str) {
    let bot = state.bot.clone();
    if let Err(e) = state.dispatcher.process(update, bot).await {
        eprintln!("[tgram webhook] handler error: {}", e);
        return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "error");
    }
    (axum::http::StatusCode::OK, "ok")
}
