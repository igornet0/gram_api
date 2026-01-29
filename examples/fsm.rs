//! FSM example: simple state flow (ask name -> greet).
//!
//! Run with: cargo run --example fsm -- <BOT_TOKEN>

use serde_json::json;
use tgram_api::prelude::*;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = env::args().nth(1).expect("Usage: fsm <BOT_TOKEN>");

    let storage = MemoryStorage::new();

    let dispatcher = Dispatcher::new()
        .with_storage(storage)
        .message(command("start"), |ctx: Context| async move {
            let chat_id = ctx.chat_id().unwrap();
            ctx.clear_state().await.ok();
            ctx.bot.send_message(chat_id, "What is your name?").await?;
            ctx.set_state(json!("waiting_name")).await?;
            Ok(())
        })
        .message(text(), |ctx: Context| async move {
            let chat_id = ctx.chat_id().unwrap();
            let state = ctx.get_state().await;
            match state.as_ref().and_then(|s| s.as_str()) {
                Some("waiting_name") => {
                    let name = ctx.message_text().unwrap_or("User").trim();
                    ctx.clear_state().await.ok();
                    ctx.bot.send_message(chat_id, &format!("Hello, {}!", name)).await?;
                }
                _ => {
                    ctx.bot.send_message(chat_id, "Send /start to begin.").await?;
                }
            }
            Ok(())
        });

    Bot::new()
        .token(&token)
        .workers(4)
        .run(dispatcher)
        .await?;

    Ok(())
}
