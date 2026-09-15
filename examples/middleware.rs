//! Middleware example: logging and throttle.
//!
//! Run with: cargo run --example middleware -- <BOT_TOKEN>

use std::env;
use gram_api::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = env::args().nth(1).expect("Usage: middleware <BOT_TOKEN>");

    let dispatcher = Dispatcher::new()
        .middleware(LoggingMiddleware::new())
        .middleware(ThrottleMiddleware::per_second(1))
        .message(text(), |ctx: Context| async move {
            let chat_id = ctx.chat_id().unwrap();
            let text = ctx.message_text().unwrap_or("");
            ctx.bot
                .send_message(chat_id, &format!("Echo: {}", text))
                .await?;
            Ok(())
        });

    Bot::new().token(&token).workers(4).run(dispatcher).await?;

    Ok(())
}
