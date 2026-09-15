//! Control-plane + API integration tests.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use clap::Parser;
use tower::ServiceExt;

use gram_api::cli::Cli;
use gram_api::control::{mask_token, redact_secrets, BotService, GramConfig, GramPaths};
use gram_api::server::{router, AppState};

fn temp_paths(label: &str) -> GramPaths {
    let root = std::env::temp_dir().join(format!(
        "gram-{label}-{}",
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
fn parses_cli_commands() {
    assert!(Cli::try_parse_from(["gram", "version"]).is_ok());
    assert!(Cli::try_parse_from(["gram", "status"]).is_ok());
    assert!(Cli::try_parse_from(["gram", "bot", "list"]).is_ok());
    assert!(Cli::try_parse_from(["gram", "bot", "add", "demo"]).is_ok());
    assert!(Cli::try_parse_from(["gram", "bot", "status", "demo"]).is_ok());
    assert!(Cli::try_parse_from(["gram", "start", "ui"]).is_ok());
    assert!(Cli::try_parse_from(["gram", "start", "ui", "--port", "9000"]).is_ok());
    assert!(Cli::try_parse_from(["gram", "--data", "/tmp/gram-test", "status"]).is_ok());
    assert!(Cli::try_parse_from(["gram", "acc", "list"]).is_ok());
    assert!(Cli::try_parse_from(["gram", "acc", "login", "+79851234567"]).is_ok());
    assert!(Cli::try_parse_from(["gram", "acc", "status", "79851234567"]).is_ok());
}

#[test]
fn mask_token_hides_secret() {
    let token = "1234567890:AAHsecretTOKEN_VALUE_HERE";
    let masked = mask_token(token);
    assert!(!masked.contains("secret"));
    assert!(!masked.contains("TOKEN_VALUE"));
    assert!(masked.contains('…'));
}

#[test]
fn redact_secrets_strips_password_and_token() {
    let line = "login password=SuperSecret123 token=1234567890:AAHxxxxxxxxxxxxxxxxxxxxFAIL";
    let red = redact_secrets(line);
    assert!(!red.contains("SuperSecret123"));
    assert!(!red.contains("FAIL"));
    assert!(red.contains("[redacted]") || red.contains('…'));
}

#[test]
fn paths_and_config_roundtrip() {
    let paths = temp_paths("cfg");
    let cfg = GramConfig::default();
    cfg.save(&paths).unwrap();
    let loaded = GramConfig::load(&paths).unwrap();
    assert_eq!(loaded.ui.port, 8788);
    assert_eq!(loaded.ui.host, "127.0.0.1");
}

#[test]
fn bot_service_add_list_remove_no_token_leak() {
    let paths = temp_paths("bots");
    let cfg = GramConfig::default();
    let bots = BotService::new(paths.clone(), cfg).unwrap();
    let secret = "1234567890:AAH-this-must-not-appear-in-meta";
    bots.add("demo", secret).unwrap();
    let list = bots.list().unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].name, "demo");
    let meta = std::fs::read_to_string(paths.bot("demo").join("bot.toml")).unwrap();
    assert!(!meta.contains(secret));
    assert!(!meta.contains("AAH-this"));
    bots.remove("demo").unwrap();
    assert!(bots.list().unwrap().is_empty());
}

#[tokio::test]
async fn api_status_bots_endpoints() {
    let paths = temp_paths("api");
    let config = GramConfig::default();
    let bots = BotService::new(paths.clone(), config.clone()).unwrap();
    bots.add("api-bot", "1234567890:AAHxxxxxxxxxxxxxxxxxxxxTEST")
        .unwrap();

    #[cfg(feature = "accounts")]
    let accounts = gram_api::control::AccountService::new(paths.clone(), config.clone()).unwrap();

    let state = Arc::new(AppState {
        paths,
        config,
        bots,
        #[cfg(feature = "accounts")]
        accounts,
    });
    let app = router(state);

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["bots"]["total"], 1);
    assert!(!body
        .windows(b"AAHxxxxxxxxxxxxxxxxxxxxTEST".len())
        .any(|w| w == b"AAHxxxxxxxxxxxxxxxxxxxxTEST"));

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/bots")
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
    assert!(text.contains("api-bot"));
    assert!(!text.contains("AAHxxxxxxxxxxxxxxxxxxxxTEST"));
}
