// Registre d'actions — native et wrapper
// Actions polymorphes : Rust natif + wrappers Python/Bash

use serde_json::Value;
use std::collections::HashMap;
use std::process::Command;

/// Signal retourné par un hook après exécution.
/// Le moteur mappe ces signaux via `on_signal` dans la config du hook.
#[derive(Debug, Clone, PartialEq)]
pub enum HookSignal {
    Continue,
    Freeze { reason: String },
    Abort { reason: String },
}

/// Contexte passé à chaque action : l'objet courant + paramètres du hook
#[derive(Debug, Clone, serde::Serialize)]
pub struct ActionContext {
    pub object_id: String,
    pub workflow_id: String,
    pub current_state: String,
    pub target_state: String,
    pub transition_id: String,
    pub payload: Value,
}

/// Interface commune à toutes les actions (natives ET wrappers)
pub trait ActionHandler: Send + Sync {
    fn name(&self) -> &str;
    fn handle(&self, ctx: &ActionContext) -> Result<HookSignal, String>;
    fn capabilities(&self) -> Vec<String> {
        vec![]
    }
}

// ─── Implémentation générique pour ScriptWrapper ────────────────────────────

pub struct ScriptWrapper {
    pub name: String,
    pub script_path: std::path::PathBuf,
    pub interpreter: String,
    pub timeout_seconds: u64,
}

impl ActionHandler for ScriptWrapper {
    fn name(&self) -> &str { &self.name }

    fn handle(&self, ctx: &ActionContext) -> Result<HookSignal, String> {
        let input = serde_json::to_string(ctx)
            .unwrap_or_else(|_| "{}".to_string());
        let output = Command::new(&self.interpreter)
            .arg(&self.script_path)
            .arg(&input)
            .output()
            .map_err(|e| format!("Wrapper {} ({}): {}", self.name, self.interpreter, e))?;
        if output.status.success() {
            Ok(HookSignal::Continue)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("Wrapper {} failed: {}", self.name, stderr))
        }
    }
}

// ─── Registre ────────────────────────────────────────────────────────────────

pub struct Registry {
    actions: HashMap<String, Box<dyn ActionHandler>>,
}

impl Registry {
    pub fn new() -> Self {
        Self { actions: HashMap::new() }
    }

    pub fn register(&mut self, action: Box<dyn ActionHandler>) {
        self.actions.insert(action.name().to_string(), action);
    }

    pub fn get(&self, name: &str) -> Option<&dyn ActionHandler> {
        self.actions.get(name).map(|b| b.as_ref())
    }

    pub fn has(&self, name: &str) -> bool {
        self.actions.contains_key(name)
    }
}
