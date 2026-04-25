// Action native : détection de loopback

use crate::registry::ActionRegistry;

pub struct LoopbackAction;

impl ActionRegistry for LoopbackAction {
    fn name(&self) -> &str {
        "detect_loopback"
    }
}
