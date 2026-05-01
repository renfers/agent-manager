// Action native : rate limiter par objet
// Stocke un compteur d'appels en mémoire (HashMap Mutex)

use crate::registry::{ActionContext, ActionHandler, HookSignal};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

pub struct RateLimitAction {
    state: Mutex<HashMap<String, (u32, Instant)>>,
    pub max_per_minute: u32,
}

impl RateLimitAction {
    pub fn new(max_per_minute: u32) -> Self {
        Self {
            state: Mutex::new(HashMap::new()),
            max_per_minute,
        }
    }
}

impl ActionHandler for RateLimitAction {
    fn name(&self) -> &str { "rate_limit" }

    fn handle(&self, ctx: &ActionContext) -> Result<HookSignal, String> {
        let per_minute = ctx.payload
            .get("max_per_minute")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32)
            .unwrap_or(self.max_per_minute);

        let key = format!("{}:{}", ctx.workflow_id, ctx.object_id);
        let now = Instant::now();

        let mut guard = self.state.lock()
            .map_err(|e| format!("RateLimit mutex: {}", e))?;

        let entry = guard.entry(key.clone()).or_insert((0, now));

        if now.duration_since(entry.1) > std::time::Duration::from_secs(60) {
            *entry = (1, now);
            Ok(HookSignal::Continue)
        } else if entry.0 >= per_minute {
            log::warn!("[rate_limiter] {} exceeded ({} per min)", key, entry.0);
            Ok(HookSignal::Freeze {
                reason: format!("{} triggered {} times in < 1 min (limit: {})", key, entry.0, per_minute),
            })
        } else {
            entry.0 += 1;
            Ok(HookSignal::Continue)
        }
    }
}
