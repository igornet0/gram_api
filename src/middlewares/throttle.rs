//! Throttle middleware: limit updates per user per second.

use crate::dispatcher::{Context, Middleware, MiddlewareError};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Middleware that throttles updates per user (e.g. 1 per second).
pub struct ThrottleMiddleware {
    min_interval: Duration,
    last_by_key: Mutex<HashMap<(i64, i64), Instant>>,
}

impl ThrottleMiddleware {
    /// Throttle to at most one update per `interval` per (user_id, chat_id).
    pub fn new(interval: Duration) -> Self {
        Self {
            min_interval: interval,
            last_by_key: Mutex::new(HashMap::new()),
        }
    }

    /// Throttle to at most `rate` updates per second per user/chat.
    pub fn per_second(rate: u64) -> Self {
        let r = rate.max(1);
        Self::new(Duration::from_millis(1000 / r))
    }
}

#[async_trait]
impl Middleware for ThrottleMiddleware {
    async fn before(&self, ctx: &mut Context) -> Result<(), MiddlewareError> {
        let key = match (ctx.user_id(), ctx.chat_id()) {
            (Some(uid), Some(cid)) => (uid, cid),
            (Some(uid), None) => (uid, 0),
            _ => return Ok(()),
        };
        let now = Instant::now();
        let mut guard = self.last_by_key.lock().map_err(|e| {
            MiddlewareError::Other(anyhow::anyhow!("lock poisoned: {}", e))
        })?;
        if let Some(&last) = guard.get(&key) {
            if now.duration_since(last) < self.min_interval {
                return Err(MiddlewareError::Other(anyhow::anyhow!(
                    "throttled: too many requests"
                )));
            }
        }
        guard.insert(key, now);
        Ok(())
    }
}
