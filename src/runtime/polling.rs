//! Long polling for updates.

use crate::bot::Bot;
use crate::core::TelegramError;
use crate::dispatcher::Dispatcher;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use tokio::sync::Semaphore;

/// Long polling runtime: fetches updates and dispatches to handlers.
pub struct Polling {
    bot: Bot,
    dispatcher: Dispatcher,
    timeout_secs: u64,
    limit: Option<u32>,
}

impl Polling {
    pub fn new(bot: Bot, dispatcher: Dispatcher) -> Self {
        Self {
            bot,
            dispatcher,
            timeout_secs: 30,
            limit: None,
        }
    }

    /// Set long polling timeout in seconds.
    pub fn timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }

    /// Set max updates per request.
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Start the polling loop.
    pub async fn start(self) -> Result<(), TelegramError> {
        let bot = Arc::new(self.bot);
        let api = bot.api.clone();
        let dispatcher = Arc::new(self.dispatcher);
        let workers = bot.config.workers;
        let semaphore = Arc::new(Semaphore::new(workers));
        let offset = Arc::new(AtomicI64::new(0));

        loop {
            let off = offset.load(Ordering::SeqCst);
            let updates = api.get_updates(off, self.timeout_secs, self.limit).await?;

            for update in updates {
                offset.store(update.update_id + 1, Ordering::SeqCst);
                let dispatcher = Arc::clone(&dispatcher);
                let bot = Arc::clone(&bot);
                let permit = semaphore.clone().acquire_owned().await.unwrap();
                tokio::spawn(async move {
                    let _permit = permit;
                    let _ = dispatcher.process(update, bot).await;
                });
            }
        }
    }
}
