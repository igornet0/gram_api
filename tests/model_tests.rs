//! Unit tests for model deserialization (serde).

use tgram_api::core::Update;

#[test]
fn test_update_deserialize_message() {
    let json = r#"{"update_id":1,"message":{"message_id":1,"date":0,"chat":{"id":2,"type":"private"},"text":"/start"}}"#;
    let update: Update = serde_json::from_str(json).unwrap();
    assert_eq!(update.update_id, 1);
    assert!(update.message.is_some());
    let msg = update.message.as_ref().unwrap();
    assert_eq!(msg.message_id, 1);
    assert_eq!(msg.text.as_deref(), Some("/start"));
    assert_eq!(msg.chat.id, 2);
}

#[test]
fn test_update_deserialize_callback_query() {
    let json = r#"{"update_id":2,"callback_query":{"id":"cq1","from":{"id":3,"is_bot":false,"first_name":"U"},"data":"ok"}}"#;
    let update: Update = serde_json::from_str(json).unwrap();
    assert_eq!(update.update_id, 2);
    assert!(update.callback_query.is_some());
    let cq = update.callback_query.as_ref().unwrap();
    assert_eq!(cq.id, "cq1");
    assert_eq!(cq.from.id, 3);
    assert_eq!(cq.data.as_deref(), Some("ok"));
}
