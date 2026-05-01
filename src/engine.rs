// Moteur de workflow — FSM générique avec hooks before/after
// Lit les 4 JSON via WorkflowConfig, exécute les actions via Registry

use crate::config::{HookDef, WorkflowConfig};
use crate::registry::{ActionContext, ActionHandler, HookSignal, Registry};
use crate::store::Store;
use std::collections::HashMap;
use std::path::PathBuf;

pub struct WorkflowEngine {
    config: WorkflowConfig,
    registry: Registry,
    store: Store,
    /// object_id → current_state
    objects: HashMap<String, String>,
    /// Marqueurs pour les états gelés (frozen)
    frozen: HashMap<String, bool>,
    dry_run: bool,
    /// Chemin racine du projet (pour trouver wrappers/)
    project_dir: PathBuf,
}

/// Résultat d'une transition exécutée
#[derive(Debug)]
pub struct TransitionResult {
    pub object_id: String,
    pub from_state: String,
    pub to_state: String,
    pub transition_id: String,
    pub hooks_fired: Vec<String>,
    pub forced_transition: Option<String>,
    pub alert_message: Option<String>,
}

impl WorkflowEngine {
    pub fn new(
        config: WorkflowConfig,
        registry: Registry,
        store: Store,
        project_dir: PathBuf,
    ) -> Self {
        let dry_run = config.app_config.dry_run;
        Self {
            config,
            registry,
            store,
            objects: HashMap::new(),
            frozen: HashMap::new(),
            dry_run,
            project_dir,
        }
    }

    /// Enregistre un nouvel objet dans le moteur avec son état initial
    pub fn register_object(&mut self, object_id: &str, initial_state: &str) -> Result<(), String> {
        if self.config.find_state(initial_state).is_none() {
            return Err(format!("État initial '{}' inconnu", initial_state));
        }
        self.objects.insert(object_id.to_string(), initial_state.to_string());
        log::info!("[engine] Objet '{}' créé → {}", object_id, initial_state);
        Ok(())
    }

    /// Retourne l'état courant d'un objet
    pub fn state_of(&self, object_id: &str) -> Option<&str> {
        self.objects.get(object_id).map(|s| s.as_str())
    }

    /// Vérifie si un objet est gelé (frozen)
    pub fn is_frozen(&self, object_id: &str) -> bool {
        self.frozen.get(object_id).copied().unwrap_or(false)
    }

    /// Dégele un objet manuellement (Anaréa uniquement)
    pub fn unfreeze(&mut self, object_id: &str) {
        self.frozen.insert(object_id.to_string(), false);
        log::info!("[engine] '{}' dégelé manuellement", object_id);
    }

    /// Exécute une transition nommée sur un objet.
    /// Retourne le résultat détaillé ou une erreur.
    pub fn apply_transition(
        &mut self,
        object_id: &str,
        transition_id: &str,
    ) -> Result<TransitionResult, String> {
        let current = self.current_state(object_id)?;

        let transition = self.config.find_transition(transition_id)
            .ok_or_else(|| format!("Transition '{}' inconnue", transition_id))?
            .clone();

        if transition.from != current {
            return Err(format!(
                "Transition '{}' attend l'état '{}', mais '{}' est en '{}'",
                transition_id, transition.from, object_id, current
            ));
        }

        self.execute_transition(object_id, &transition)
    }

    /// Trouve et exécute la première transition automatique disponible depuis l'état courant
    pub fn run_auto(&mut self, object_id: &str) -> Result<Option<TransitionResult>, String> {
        let current = self.current_state(object_id)?;

        let candidates: Vec<_> = self.config.transitions.iter()
            .filter(|t| t.from == current && t.trigger == "automatic")
            .collect();

        if candidates.is_empty() {
            log::debug!("[engine] '{}' en '{}' : aucune transition auto", object_id, current);
            return Ok(None);
        }

        let transition = candidates[0].clone();
        let result = self.execute_transition(object_id, &transition)?;
        Ok(Some(result))
    }

    /// Accès lecture à la config
    pub fn config(&self) -> &WorkflowConfig { &self.config }

    /// Nombre d'objets suivis
    pub fn state_count(&self) -> usize { self.objects.len() }

    /// Mode dry-run actif?
    pub fn dry_run(&self) -> bool { self.dry_run }

    /// Trouve les transitions possibles depuis l'état courant (pour affichage/UI)
    pub fn available_transitions(&self, object_id: &str) -> Result<Vec<String>, String> {
        let current = self.current_state(object_id)?;
        Ok(self.config.transitions.iter()
            .filter(|t| t.from == current)
            .map(|t| t.id.clone())
            .collect())
    }

    // ─── interne ────────────────────────────────────────────────────────────

    fn current_state(&self, object_id: &str) -> Result<String, String> {
        if self.frozen.get(object_id).copied().unwrap_or(false) {
            return Err(format!("Objet '{}' est gelé (frozen). /unfreeze requis.", object_id));
        }
        self.objects.get(object_id)
            .cloned()
            .ok_or_else(|| format!("Objet '{}' inconnu", object_id))
    }

    fn execute_transition(
        &mut self,
        object_id: &str,
        transition: &crate::config::TransitionDef,
    ) -> Result<TransitionResult, String> {
        let from_state = self.objects[object_id].clone();
        let mut hooks_fired: Vec<String> = vec![];
        let mut forced: Option<String> = None;
        let mut alert: Option<String> = None;

        // ── Before hooks ──────────────────────────────────────────────────
        for hook in self.config.find_hooks(&transition.id, "before") {
            let signal = self.fire_hook(hook, object_id, &from_state, &transition.to, &transition.id)?;
            hooks_fired.push(hook.hook_id.clone());
            match signal {
                HookSignal::Continue => continue,
                HookSignal::Freeze { ref reason } | HookSignal::Abort { ref reason } => {
                    let reason = reason.clone();
                    log::warn!("[engine] Hook {} returned signal: {}",
                        hook.hook_id, reason);
                    if let Some(mapping) = hook.on_signal.get("Freeze")
                        .or_else(|| hook.on_signal.get("Abort"))
                    {
                        if let Some(alt_transition) = &mapping.transition {
                            forced = Some(alt_transition.clone());
                            log::info!("[engine] ↳ Forcé vers transition '{}'", alt_transition);
                            if matches!(signal, HookSignal::Freeze { .. }) {
                                self.frozen.insert(object_id.to_string(), true);
                            }
                            if mapping.alert_anarea {
                                alert = mapping.message.clone()
                                    .or_else(|| Some(reason.clone()));
                            }
                            return Ok(TransitionResult {
                                object_id: object_id.to_string(),
                                from_state: from_state.clone(),
                                to_state: from_state.clone(),
                                transition_id: transition.id.clone(),
                                hooks_fired,
                                forced_transition: forced.clone(),
                                alert_message: alert.clone(),
                            });
                        }
                        if mapping.alert_anarea {
                            alert = mapping.message.clone().or_else(|| Some(reason.clone()));
                        }
                    }
                    // Si pas de on_signal, abort = erreur
                    if hook.on_error == "abort" && matches!(signal, HookSignal::Abort { .. }) {
                        return Err(format!("Hook {} aborted: {}", hook.hook_id, reason));
                    }
                    // continue = on ignore le signal
                    if hook.on_error == "continue" {
                        continue;
                    }
                }
            }
        }

        // ── Changer l'état ────────────────────────────────────────────────
        let to_state = forced.as_deref().unwrap_or(&transition.to).to_string();
        self.objects.insert(object_id.to_string(), to_state.clone());

        if self.dry_run {
            log::info!("[engine] DRY-RUN {}: {} → {} ({})",
                object_id, from_state, to_state, transition.id);
        } else {
            let _ = self.store.log_transition(
                &self.config.name,
                object_id,
                &from_state,
                &to_state,
            );
        }

        // ── After hooks ───────────────────────────────────────────────────
        for hook in self.config.find_hooks(&transition.id, "after") {
            let signal = self.fire_hook(hook, object_id, &from_state, &to_state, &transition.id)?;
            hooks_fired.push(hook.hook_id.clone());
            match signal {
                HookSignal::Continue => continue,
                HookSignal::Freeze { ref reason } | HookSignal::Abort { ref reason } => {
                    let reason = reason.clone();
                    if let Some(mapping) = hook.on_signal.get("Freeze")
                        .or_else(|| hook.on_signal.get("Abort"))
                    {
                        if mapping.alert_anarea {
                            alert = mapping.message.clone().or_else(|| Some(reason));
                        }
                    }
                }
            }
        }

        log::info!("[engine] {}: {} → {} via {}", object_id, from_state, to_state, transition.id);

        Ok(TransitionResult {
            object_id: object_id.to_string(),
            from_state,
            to_state,
            transition_id: transition.id.clone(),
            hooks_fired,
            forced_transition: forced,
            alert_message: alert,
        })
    }

    fn fire_hook(
        &self,
        hook: &HookDef,
        object_id: &str,
        current_state: &str,
        target_state: &str,
        transition_id: &str,
    ) -> Result<HookSignal, String> {
        if self.dry_run && hook.action == "send_telegram" {
            log::info!("[engine] DRY-RUN hook {} ({}) — skip Telegram", hook.hook_id, hook.action);
            return Ok(HookSignal::Continue);
        }

        let ctx = ActionContext {
            object_id: object_id.to_string(),
            workflow_id: self.config.name.clone(),
            current_state: current_state.to_string(),
            target_state: target_state.to_string(),
            transition_id: transition_id.to_string(),
            payload: hook.payload.clone(),
        };

        match self.registry.get(&hook.action) {
            Some(handler) => {
                log::debug!("[engine] Hook {} → action {}", hook.hook_id, hook.action);
                handler.handle(&ctx)
            }
            None => {
                let wrapper_path = self.project_dir.join("wrappers");
                self.try_wrapper(&wrapper_path, &hook.action, &ctx)
            }
        }
    }

    fn try_wrapper(
        &self,
        wrappers_dir: &std::path::Path,
        action_name: &str,
        ctx: &ActionContext,
    ) -> Result<HookSignal, String> {
        // Cherche un wrapper Python ou Bash correspondant
        for ext in &["py", "sh"] {
            let script = wrappers_dir.join(format!("{}.{}", action_name, ext));
            if script.exists() {
                let interpreter = if *ext == "py" { "python3" } else { "bash" };
                let wrapper = crate::registry::ScriptWrapper {
                    name: action_name.to_string(),
                    script_path: script,
                    interpreter: interpreter.to_string(),
                    timeout_seconds: 60,
                };
                return wrapper.handle(ctx);
            }
        }
        Err(format!("Action '{}' introuvable dans le registre et les wrappers", action_name))
    }
}
