// Lecture et validation des 4 JSON de workflow (spec v2)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

// ─── States ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StateDef {
    pub id: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_state_type")]
    pub r#type: String,         // stable, transient, paused, terminal
    #[serde(default)]
    pub auto_exit: bool,
    pub max_duration: Option<String>,
    pub on_timeout: Option<String>,
    #[serde(default)]
    pub requires_manual_unlock: bool,
}

fn default_state_type() -> String { "stable".into() }

// ─── Transitions ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TransitionDef {
    pub id: String,
    pub from: String,
    pub to: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_trigger")]
    pub trigger: String,        // external, automatic, hook_decision, manual
    pub permissions: Option<Vec<String>>,
    pub condition: Option<String>,
}

fn default_trigger() -> String { "external".into() }

// ─── Hooks ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HookDef {
    pub hook_id: String,
    pub transition_id: String,
    pub timing: String,         // before, after
    #[serde(default = "default_priority")]
    pub priority: u32,
    pub action: String,
    #[serde(default)]
    pub payload: serde_json::Value,
    #[serde(default = "default_on_error")]
    pub on_error: String,       // continue, abort, retry
    #[serde(default)]
    pub on_signal: HashMap<String, SignalMapping>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct SignalMapping {
    pub transition: Option<String>,
    #[serde(default)]
    pub alert_anarea: bool,
    pub message: Option<String>,
}

fn default_priority() -> u32 { 1 }
fn default_on_error() -> String { "abort".into() }

// ─── Config (le 4ème JSON) ───────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    #[serde(default)]
    pub dry_run: bool,

    pub moteur: Option<EngineConfig>,

    pub rate_limiter: Option<RateLimiterConfig>,

    pub chatroom_bot: Option<ChatroomBotConfig>,

    pub anarea: Option<UserConfig>,

    #[serde(default)]
    pub presences: HashMap<String, PresenceConfig>,

    pub registry: Option<ActionRegistryConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EngineConfig {
    pub journal_db: Option<String>,
    pub flush_interval_seconds: Option<u64>,
    pub max_history_per_object: Option<usize>,
    pub thread_pool_size: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RateLimiterConfig {
    pub max_public_per_minute: Option<u32>,
    pub cooldown_minutes: Option<u32>,
    pub loopback_window_minutes: Option<u64>,
    pub loopback_threshold: Option<usize>,
    pub rapid_chain_threshold: Option<u32>,
    pub rapid_chain_window_seconds: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChatroomBotConfig {
    pub token_env: Option<String>,
    pub username: Option<String>,
    pub bot_user_id: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserConfig {
    pub name: String,
    pub icon: String,
    pub telegram_user_id: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PresenceConfig {
    pub name: String,
    pub icon: String,
    pub bot_user_id: Option<i64>,
    pub username: Option<String>,
    pub model: Option<String>,
    #[serde(default = "default_true")]
    pub dm_enabled: bool,
    pub note: Option<String>,
}

fn default_true() -> bool { true }

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ActionRegistryConfig {
    #[serde(default)]
    pub native_actions: Vec<NativeActionConfig>,
    #[serde(default)]
    pub wrapper_actions: Vec<WrapperActionConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NativeActionConfig {
    pub name: String,
    pub r#type: String,
    pub config: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WrapperActionConfig {
    pub name: String,
    pub interpreter: String,
    pub script: String,
    #[serde(default)]
    pub timeout_seconds: u64,
}

// ─── Workflow complet ────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct WorkflowConfig {
    pub name: String,
    pub dir: std::path::PathBuf,
    pub states: Vec<StateDef>,
    pub transitions: Vec<TransitionDef>,
    pub hooks: Vec<HookDef>,
    pub app_config: AppConfig,
}

impl WorkflowConfig {
    pub fn load(dir: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let name = dir.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let states: Vec<StateDef> = Self::read_json(dir, "states.json")?;
        let transitions: Vec<TransitionDef> = Self::read_json(dir, "transitions.json")?;
        let hooks: Vec<HookDef> = Self::read_json(dir, "hooks.json")?;
        let app_config: AppConfig = Self::read_json(dir, "config.json")?;

        // Validation : tous les from/to existent dans states
        let state_ids: Vec<&str> = states.iter().map(|s| s.id.as_str()).collect();
        for t in &transitions {
            if !state_ids.contains(&t.from.as_str()) {
                return Err(format!(
                    "Transition '{}': from '{}' inconnu dans states.json", t.id, t.from
                ).into());
            }
            if !state_ids.contains(&t.to.as_str()) {
                return Err(format!(
                    "Transition '{}': to '{}' inconnu dans states.json", t.id, t.to
                ).into());
            }
        }

        // Validation : tous les transition_id des hooks existent
        let transition_ids: Vec<&str> = transitions.iter().map(|t| t.id.as_str()).collect();
        for h in &hooks {
            if !transition_ids.contains(&h.transition_id.as_str()) {
                return Err(format!(
                    "Hook '{}': transition_id '{}' inconnu dans transitions.json",
                    h.hook_id, h.transition_id
                ).into());
            }
        }

        // Validation : pas de transitions depuis un état terminal
        for t in &transitions {
            if let Some(state) = states.iter().find(|s| s.id == t.from) {
                if state.r#type == "terminal" {
                    return Err(format!(
                        "Transition '{}' part d'un état terminal '{}'", t.id, t.from
                    ).into());
                }
            }
        }

        log::info!("[config] {} chargé : {} états, {} transitions, {} hooks",
            name, states.len(), transitions.len(), hooks.len());

        Ok(Self {
            name,
            dir: dir.to_path_buf(),
            states,
            transitions,
            hooks,
            app_config,
        })
    }

    fn read_json<T: serde::de::DeserializeOwned>(
        dir: &Path,
        filename: &str,
    ) -> Result<T, Box<dyn std::error::Error>> {
        let path = dir.join(filename);
        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("Cannot read {}: {}", path.display(), e))?;
        let value: T = serde_json::from_str(&content)
            .map_err(|e| format!("Invalid JSON in {}: {}", path.display(), e))?;
        Ok(value)
    }

    pub fn find_state(&self, id: &str) -> Option<&StateDef> {
        self.states.iter().find(|s| s.id == id)
    }

    pub fn find_transitions_from(&self, state_id: &str) -> Vec<&TransitionDef> {
        self.transitions.iter()
            .filter(|t| t.from == state_id)
            .collect()
    }

    pub fn find_transition(&self, id: &str) -> Option<&TransitionDef> {
        self.transitions.iter().find(|t| t.id == id)
    }

    pub fn find_hooks(&self, transition_id: &str, timing: &str) -> Vec<&HookDef> {
        let mut hooks: Vec<&HookDef> = self.hooks.iter()
            .filter(|h| h.transition_id == transition_id && h.timing == timing)
            .collect();
        hooks.sort_by_key(|h| h.priority);
        hooks
    }
}
