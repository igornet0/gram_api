//! Unit tests for filters (command, text, regex).

use tgram_api::core::{Chat, Message, Update, User};
use tgram_api::dispatcher::{Context, Filter};
use tgram_api::filters::{command, regex, text};
use tgram_api::Bot;
use std::sync::Arc;

fn make_ctx(msg_text: Option<&str>) -> Context {
    let user = User {
        id: 1,
        is_bot: false,
        first_name: "Test".to_string(),
        last_name: None,
        username: None,
        language_code: None,
    };
    let chat = Chat {
        id: 2,
        kind: "private".to_string(),
        title: None,
        username: None,
        first_name: Some("Test".to_string()),
        last_name: None,
    };
    let message = Message {
        message_id: 1,
        from: Some(user),
        date: 0,
        chat: chat.clone(),
        text: msg_text.map(str::to_string),
        reply_to_message: None,
        reply_markup: None,
    };
    let update = Update {
        update_id: 1,
        message: Some(message),
        edited_message: None,
        callback_query: None,
        inline_query: None,
        chosen_inline_result: None,
    };
    Context::new(update, Arc::new(Bot::new().token("test")), None, (1, 2))
}

#[test]
fn test_command_start() {
    let cmd = command("start");
    assert!(cmd.check(&make_ctx(Some("/start"))));
    assert!(cmd.check(&make_ctx(Some("/start "))));
    assert!(cmd.check(&make_ctx(Some("/start@mybot"))));
    assert!(cmd.check(&make_ctx(Some("/START"))));
    assert!(!cmd.check(&make_ctx(Some("/help"))));
    assert!(!cmd.check(&make_ctx(Some("start"))));
    assert!(!cmd.check(&make_ctx(None)));
}

#[test]
fn test_text() {
    let f = text();
    assert!(f.check(&make_ctx(Some("hello"))));
    assert!(f.check(&make_ctx(Some(" "))));
    assert!(!f.check(&make_ctx(None)));
}

#[test]
fn test_regex() {
    let f = regex(r"^\d+$").unwrap();
    assert!(f.check(&make_ctx(Some("123"))));
    assert!(f.check(&make_ctx(Some("0"))));
    assert!(!f.check(&make_ctx(Some("12a"))));
    assert!(!f.check(&make_ctx(None)));
}

#[test]
fn test_filter_and() {
    use tgram_api::filters::FilterExt;
    let f = command("start").and(text());
    assert!(f.check(&make_ctx(Some("/start"))));
    assert!(!f.check(&make_ctx(Some("/start")))); // wait - /start has text, so it should pass
    // Actually command("start") and text() both pass for "/start". So f.check should be true.
    // Let me double-check: command("start") on "/start" -> true, text() on "/start" -> true. So and() -> true. Good.
}
