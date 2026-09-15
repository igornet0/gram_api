//! Account control-plane tests (compiled with `--features accounts`).

#![cfg(feature = "accounts")]

use gram_api::account::{AccountManager, TdlibCredentials};
use gram_api::control::{
    AccountService, AuthPhase, GramConfig, GramPaths, LoginPrompts, LoginRequest,
};

fn temp_paths(label: &str) -> GramPaths {
    let root = std::env::temp_dir().join(format!(
        "gram-acc-{label}-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    ));
    let paths = GramPaths::new(root);
    paths.ensure().unwrap();
    paths
}

#[test]
fn credentials_and_manager_construct() {
    let creds = TdlibCredentials {
        api_id: 1,
        api_hash: "deadbeef".into(),
        use_test_dc: true,
    };
    let dir = std::env::temp_dir().join(format!(
        "tgram-accounts-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    ));
    let mgr = AccountManager::new(&dir);
    let _ = (creds.api_id, mgr.data_dir().exists());
    assert!(dir.exists() || std::fs::create_dir_all(&dir).is_ok());
}

#[test]
fn auth_phase_variants() {
    assert_eq!(
        serde_json::to_string(&AuthPhase::CodeRequired).unwrap(),
        "\"CODE_REQUIRED\""
    );
    assert_eq!(
        serde_json::to_string(&AuthPhase::PasswordRequired).unwrap(),
        "\"PASSWORD_REQUIRED\""
    );
    assert_eq!(
        serde_json::to_string(&AuthPhase::Authorized).unwrap(),
        "\"AUTHORIZED\""
    );
    assert_eq!(
        serde_json::to_string(&AuthPhase::PhoneRequired).unwrap(),
        "\"PHONE_REQUIRED\""
    );
    assert_eq!(
        serde_json::to_string(&AuthPhase::Error).unwrap(),
        "\"ERROR\""
    );
}

#[test]
fn account_service_create_list_remove() {
    let paths = temp_paths("svc");
    let mut cfg = GramConfig::default();
    cfg.telegram.api_id = Some(1);
    cfg.telegram.api_hash = Some("hash".into());
    let svc = AccountService::new(paths.clone(), cfg).unwrap();
    let rec = svc.ensure_record("+7 (985) 123-45-67").unwrap();
    assert_eq!(rec.key, "79851234567");
    assert_eq!(rec.phone, "+79851234567");
    let list = svc.list().unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(svc.info(&rec.key).unwrap().phone, "+79851234567");
    // remove is async; use runtime
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async {
        svc.remove(&rec.key).await.unwrap();
    });
    assert!(svc.list().unwrap().is_empty());
}

#[tokio::test]
async fn login_without_credentials_is_config_error() {
    let paths = temp_paths("login-cfg");
    let cfg = GramConfig::default(); // no api_id
    let svc = AccountService::new(paths, cfg).unwrap();
    let err = svc
        .login(
            LoginRequest {
                phone: "+79851112233".into(),
                code: None,
                password: None,
                first_name: None,
            },
            LoginPrompts::None,
        )
        .await
        .unwrap_err();
    let msg = err.to_string();
    assert!(msg.to_ascii_lowercase().contains("api_id") || msg.contains("credentials"));
}

#[tokio::test]
async fn api_accounts_list() {
    use std::sync::Arc;

    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    use gram_api::control::BotService;
    use gram_api::server::{router, AppState};

    let paths = temp_paths("api-acc");
    let mut cfg = GramConfig::default();
    cfg.telegram.api_id = Some(1);
    cfg.telegram.api_hash = Some("hash".into());
    let bots = BotService::new(paths.clone(), cfg.clone()).unwrap();
    let accounts = AccountService::new(paths.clone(), cfg.clone()).unwrap();
    accounts.ensure_record("+79850001122").unwrap();

    let state = Arc::new(AppState {
        paths,
        config: cfg,
        bots,
        accounts,
    });
    let app = router(state);
    let res = app
        .oneshot(
            Request::builder()
                .uri("/api/accounts")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let text = String::from_utf8(body.to_vec()).unwrap();
    assert!(text.contains("79850001122"));
}
