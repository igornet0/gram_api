//! Commands bot: /start and /help with filters.
//!
//! Run with: cargo run --example commands -- <BOT_TOKEN>

use tgram_api::prelude::*;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = env::args().nth(1).expect("Usage: commands <BOT_TOKEN>");

    let dispatcher = Dispatcher::new()
        .message(command("start"), |ctx: Context| async move {
            let chat_id = ctx.chat_id().unwrap();
            ctx.bot.send_message(chat_id, "Hello! Send /help for commands.").await?;
            Ok(())
        })
        .message(command("help"), |ctx: Context| async move {
            let chat_id = ctx.chat_id().unwrap();
            ctx.bot.send_message(
                chat_id,
                "Commands:\n/start - Start\n/help - This message",
            ).await?;
            Ok(())
        })
        .message(text(), |ctx: Context| async move {
            let chat_id = ctx.chat_id().unwrap();
            ctx.bot.send_message(chat_id, "Unknown command. Send /help.").await?;
            Ok(())
        });

    Bot::new()
        .token(&token)
        .workers(4)
        .run(dispatcher)
        .await?;

    Ok(())
}
