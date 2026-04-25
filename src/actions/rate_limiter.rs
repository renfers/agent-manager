// Action native : rate limiter

use crate::registry::ActionRegistry;

pub struct RateLimitAction;

impl ActionRegistry for RateLimitAction {
    fn name(&self) -> &str {
        "rate_limit"
    }
}
