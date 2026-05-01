// Action native : détection de boucles (loopback)
// Détecte si une même transition est répétée trop de fois sur un objet

use crate::registry::{ActionContext, ActionHandler, HookSignal};
use std::collections::HashMap;
use std::sync::Mutex;

pub struct LoopbackDetector {
    state: Mutex<HashMap<String, Vec<String>>>,
    pub threshold: usize,
    pub window_minutes: u64,
}

impl LoopbackDetector {
    pub fn new(threshold: usize) -> Self {
        Self {
            state: Mutex::new(HashMap::new()),
            threshold,
            window_minutes: 60,
        }
    }
}

impl ActionHandler for LoopbackDetector {
    fn name(&self) -> &str { "detect_loopback" }

    fn handle(&self, ctx: &ActionContext) -> Result<HookSignal, String> {
        let threshold = ctx.payload
            .get("threshold")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize)
            .unwrap_or(self.threshold);

        // On détecte une boucle quand la même transition (from→to) est répétée
        let key = format!("{}:{}", ctx.workflow_id, ctx.object_id);
        let pattern = format!("{}→{}", ctx.current_state, ctx.target_state);

        let mut guard = self.state.lock()
            .map_err(|e| format!("Loopback mutex: {}", e))?;

        let entry = guard.entry(key.clone()).or_default();

        // Garder seulement les N dernières transitions
        entry.push(pattern.clone());
        if entry.len() > threshold * 3 {
            let start = entry.len().saturating_sub(threshold * 3);
            *entry = entry[start..].to_vec();
        }

        // Compter les occurrences du pattern dans la fenêtre
        let recent: Vec<_> = entry.iter()
            .rev()
            .take(threshold)
            .collect();

        let all_same = recent.len() >= threshold
            && recent.iter().all(|p| *p == &pattern);

        if all_same {
            log::warn!("[loopback] {} — {} répété {} fois", key, pattern, threshold);
            return Ok(HookSignal::Abort {
                reason: format!("Loopback détecté : {} → répété {} fois", pattern, threshold),
            });
        }

        Ok(HookSignal::Continue)
    }
}
