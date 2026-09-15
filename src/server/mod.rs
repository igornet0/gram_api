//! Local HTTP API + minimal embedded Web UI for `gram start ui`.

use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::{Path, State};
use axum::response::{Html, IntoResponse, Json, Response};
use axum::routing::{delete, get, post};
use axum::Router;
use serde::Deserialize;
use serde_json::json;
use tower_http::cors::{Any, CorsLayer};

use crate::control::{BotService, ControlError, GramConfig, GramPaths};

#[cfg(feature = "accounts")]
use crate::control::{AccountService, LoginRequest};

pub struct AppState {
    pub paths: GramPaths,
    pub config: GramConfig,
    pub bots: BotService,
    #[cfg(feature = "accounts")]
    pub accounts: AccountService,
}

pub async fn serve(state: Arc<AppState>, host: &str, port: u16) -> Result<(), ControlError> {
    let app = router(state);
    let addr: SocketAddr = format!("{host}:{port}")
        .parse()
        .map_err(|e| ControlError::Invalid(format!("invalid bind address: {e}")))?;
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| ControlError::Io(e.to_string()))?;
    axum::serve(listener, app)
        .await
        .map_err(|e| ControlError::Io(e.to_string()))?;
    Ok(())
}

pub fn router(state: Arc<AppState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let api = Router::new()
        .route("/api/status", get(api_status))
        .route("/api/bots", get(api_bots_list).post(api_bots_add))
        .route("/api/bots/:name", delete(api_bots_remove))
        .route("/api/bots/:name/start", post(api_bots_start))
        .route("/api/bots/:name/stop", post(api_bots_stop))
        .route("/api/bots/:name/status", get(api_bots_status));

    #[cfg(feature = "accounts")]
    let api = {
        api.route("/api/accounts", get(api_accounts_list))
            .route("/api/accounts/login", post(api_accounts_login))
            .route(
                "/api/accounts/:id",
                get(api_accounts_get).delete(api_accounts_remove),
            )
            .route("/api/accounts/:id/start", post(api_accounts_start))
            .route("/api/accounts/:id/stop", post(api_accounts_stop))
            .route("/api/accounts/:id/logout", post(api_accounts_logout))
            .route("/api/accounts/:id/status", get(api_accounts_status))
    };

    Router::new()
        .route("/", get(ui_index))
        .merge(api)
        .layer(cors)
        .with_state(state)
}

fn err_json(err: ControlError) -> Response {
    let status = match &err {
        ControlError::NotFound(_) => axum::http::StatusCode::NOT_FOUND,
        ControlError::Auth(_) => axum::http::StatusCode::UNAUTHORIZED,
        ControlError::Invalid(_) | ControlError::Config(_) => axum::http::StatusCode::BAD_REQUEST,
        _ => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
    };
    (status, Json(json!({ "error": err.to_string() }))).into_response()
}

async fn ui_index() -> Html<&'static str> {
    Html(include_str!("ui.html"))
}

async fn api_status(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let bots = state.bots.list().unwrap_or_default();
    let running = bots.iter().filter(|b| b.desired_running).count();
    #[cfg(feature = "accounts")]
    let accounts = state.accounts.list().unwrap_or_default();
    #[cfg(not(feature = "accounts"))]
    let accounts: Vec<()> = vec![];

    Json(json!({
        "gram": env!("CARGO_PKG_VERSION"),
        "features": {
            "accounts": cfg!(feature = "accounts"),
            "cli": true,
        },
        "bots": { "total": bots.len(), "running": running, "stopped": bots.len() - running },
        "accounts": { "total": accounts.len() },
        "ui": { "host": state.config.ui.host, "port": state.config.ui.port },
        "data_dir": state.paths.root().display().to_string(),
    }))
}

async fn api_bots_list(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.bots.list() {
        Ok(list) => Json(json!({ "bots": list })).into_response(),
        Err(e) => err_json(e).into_response(),
    }
}

#[derive(Deserialize)]
struct AddBotBody {
    name: String,
    token: String,
}

async fn api_bots_add(
    State(state): State<Arc<AppState>>,
    Json(body): Json<AddBotBody>,
) -> impl IntoResponse {
    match state.bots.add(&body.name, &body.token) {
        Ok(rec) => Json(json!({ "bot": rec })).into_response(),
        Err(e) => err_json(e).into_response(),
    }
}

async fn api_bots_remove(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    match state.bots.remove(&name) {
        Ok(()) => Json(json!({ "ok": true })).into_response(),
        Err(e) => err_json(e).into_response(),
    }
}

async fn api_bots_start(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    match state.bots.start(&name).await {
        Ok(s) => Json(json!({ "status": s })).into_response(),
        Err(e) => err_json(e).into_response(),
    }
}

async fn api_bots_stop(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    match state.bots.stop(&name).await {
        Ok(s) => Json(json!({ "status": s })).into_response(),
        Err(e) => err_json(e).into_response(),
    }
}

async fn api_bots_status(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    match state.bots.status(&name).await {
        Ok(s) => Json(json!({ "status": s })).into_response(),
        Err(e) => err_json(e).into_response(),
    }
}

#[cfg(feature = "accounts")]
async fn api_accounts_list(State(state): State<Arc<AppState>>) -> Response {
    match state.accounts.list() {
        Ok(list) => Json(json!({ "accounts": list })).into_response(),
        Err(e) => err_json(e),
    }
}

#[cfg(feature = "accounts")]
#[derive(Deserialize)]
struct LoginBody {
    phone: String,
    code: Option<String>,
    password: Option<String>,
    first_name: Option<String>,
}

#[cfg(feature = "accounts")]
async fn api_accounts_login(
    State(state): State<Arc<AppState>>,
    Json(body): Json<LoginBody>,
) -> Response {
    let req = LoginRequest {
        phone: body.phone,
        code: body.code,
        password: body.password,
        first_name: body.first_name,
    };
    match state
        .accounts
        .login(req, crate::control::LoginPrompts::None)
        .await
    {
        Ok(res) => Json(json!({
            "key": res.key,
            "phase": res.phase,
            "message": res.message,
            "account": res.view,
        }))
        .into_response(),
        Err(e) => err_json(e),
    }
}

#[cfg(feature = "accounts")]
async fn api_accounts_get(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Response {
    match state.accounts.info(&id) {
        Ok(rec) => Json(json!({ "account": rec })).into_response(),
        Err(e) => err_json(e),
    }
}

#[cfg(feature = "accounts")]
async fn api_accounts_status(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Response {
    match state.accounts.status(&id).await {
        Ok(v) => Json(json!({ "status": v })).into_response(),
        Err(e) => err_json(e),
    }
}

#[cfg(feature = "accounts")]
async fn api_accounts_start(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Response {
    match state.accounts.start(&id).await {
        Ok(v) => Json(json!({ "status": v })).into_response(),
        Err(e) => err_json(e),
    }
}

#[cfg(feature = "accounts")]
async fn api_accounts_stop(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Response {
    match state.accounts.stop(&id).await {
        Ok(()) => Json(json!({ "ok": true })).into_response(),
        Err(e) => err_json(e),
    }
}

#[cfg(feature = "accounts")]
async fn api_accounts_logout(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Response {
    match state.accounts.logout(&id).await {
        Ok(()) => Json(json!({ "ok": true })).into_response(),
        Err(e) => err_json(e),
    }
}

#[cfg(feature = "accounts")]
async fn api_accounts_remove(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Response {
    match state.accounts.remove(&id).await {
        Ok(()) => Json(json!({ "ok": true })).into_response(),
        Err(e) => err_json(e),
    }
}
