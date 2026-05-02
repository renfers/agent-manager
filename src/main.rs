use std::path::PathBuf;
use clap::Parser;

mod engine;
mod registry;
mod config;
mod store;
mod actions;
mod objects;

use config::WorkflowConfig;
use registry::Registry;
use store::Store;
use engine::WorkflowEngine;
use actions::{SendTelegramAction, RateLimitAction, LoopbackDetector};

#[derive(Parser)]
#[command(name = "agent-manager")]
#[command(about = "Moteur de workflow universel pour la constellation Anaréa")]
struct Cli {
    /// Workflow à exécuter (nom du dossier dans registry/)
    #[arg(short, long)]
    workflow: String,

    /// Mode dry-run : logue mais n'exécute aucune action externe
    #[arg(long)]
    dry_run: bool,

    /// Chemin vers le dossier registry
    #[arg(short, long, default_value = "registry")]
    registry: PathBuf,

    /// Objet à manipuler (pour les commandes ponctuelles)
    #[arg(short, long)]
    object: Option<String>,

    /// Transition à appliquer (avec --object)
    #[arg(short, long)]
    transition: Option<String>,

    /// Payload JSON runtime (fusionné dans les hooks)
    #[arg(short, long, default_value = "{}")]
    payload: String,

    /// État initial pour un nouvel objet
    #[arg(long, default_value = "idéation")]
    initial_state: String,

    /// Mode démo : crée un objet et traverse toutes les transitions auto
    #[arg(long)]
    demo: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let cli = Cli::parse();

    // Résoudre le chemin du workflow
    let workflow_dir = cli.registry.join(&cli.workflow);
    if !workflow_dir.exists() {
        let alt = PathBuf::from("/usr/local/share/agent-manager/registry").join(&cli.workflow);
        if !alt.exists() {
            return Err(format!(
                "Workflow '{}' introuvable dans {} ou {}",
                cli.workflow, workflow_dir.display(), alt.display()
            ).into());
        }
    }

    // 1. Charger la config
    let mut config = WorkflowConfig::load(&workflow_dir)?;
    if cli.dry_run {
        config.app_config.dry_run = true;
    }

    // 2. Construire le registre d'actions
    let mut registry = Registry::new();

    let telegram_token = std::env::var("CHATROOM_BOT_TOKEN")
        .unwrap_or_else(|_| "NO_TOKEN".to_string());
    let anarea_chat_id = config.app_config.anarea.as_ref()
        .and_then(|a| a.telegram_user_id)
        .map(|id| id.to_string())
        .unwrap_or_else(|| "1047918071".to_string());

    let rate_limit_max = config.app_config.rate_limiter.as_ref()
        .and_then(|r| r.max_public_per_minute)
        .unwrap_or(5);

    let loopback_threshold = config.app_config.rate_limiter.as_ref()
        .and_then(|r| r.loopback_threshold)
        .unwrap_or(3);

    registry.register(Box::new(SendTelegramAction::new(telegram_token, anarea_chat_id)));
    registry.register(Box::new(RateLimitAction::new(rate_limit_max)));
    registry.register(Box::new(LoopbackDetector::new(loopback_threshold)));

    // 3. Ouvrir le store
    let db_path = config.app_config.moteur.as_ref()
        .and_then(|m| m.journal_db.as_deref())
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join(format!("agent-manager-{}.db", config.name)));
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let store = Store::open(&db_path)?;

    // 4. Créer le moteur
    let project_dir = std::env::current_dir()?;
    let mut engine = WorkflowEngine::new(config.clone(), registry, store, project_dir);

    // 5. Exécution
    if cli.demo {
        let demo_id = "demo-001";
        engine.register_object(demo_id, "idéation")?;
        println!("═══ Démo workflow — {} ═══\n", config.name);

        let path = vec!["semer", "ouvrir", "démarrer", "terminer", "archiver"];
        for transition_id in &path {
            print!("{} → ", engine.state_of(demo_id).unwrap_or("?"));
            match engine.apply_transition(demo_id, transition_id) {
                Ok(result) => {
                    println!("{} via {}", result.to_state, result.transition_id);
                    if let Some(ref alert) = result.alert_message {
                        println!("  ⚠️  Alerte: {}", alert);
                    }
                    if let Some(ref forced) = result.forced_transition {
                        println!("  🔀 Forcé: {}", forced);
                    }
                }
                Err(e) => {
                    println!("\n  ❌ {}", e);
                }
            }
        }
        println!("\n✅ Cycle complet : {} → {}", "idéation", engine.state_of(demo_id).unwrap_or("?"));
    } else if let Some(ref object_id) = cli.object {
        if let Some(ref transition_id) = cli.transition {
            let runtime_payload: serde_json::Value = serde_json::from_str(&cli.payload)
                .unwrap_or_else(|_| serde_json::Value::Object(Default::default()));
            match engine.apply_transition_with_payload(object_id, transition_id, runtime_payload) {
                Ok(result) => {
                    println!("✅ {} : {} → {} via {}",
                        result.object_id, result.from_state, result.to_state, result.transition_id);
                    println!("   Hooks: {:?}", result.hooks_fired);
                    if let Some(ref alert) = result.alert_message {
                        println!("   ⚠️  Alerte: {}", alert);
                    }
                    if let Some(ref forced) = result.forced_transition {
                        println!("   🔀 Transition forcée: {}", forced);
                    }
                }
                Err(e) => {
                    eprintln!("❌ Erreur: {}", e);
                    std::process::exit(1);
                }
            }
        } else {
            engine.register_object(object_id, &cli.initial_state)?;
            println!("✅ Objet '{}' créé → {}", object_id, cli.initial_state);
        }
    } else {
        println!("═══ agent-manager — {} ═══", cli.workflow);
        println!();
        println!("États ({}):", config.states.len());
        for state in &config.states {
            println!("  • {} ({})", state.id, state.r#type);
        }
        println!();
        println!("Transitions ({}):", config.transitions.len());
        for t in &config.transitions {
            println!("  • {} : {} → {} [{}]", t.id, t.from, t.to, t.trigger);
        }
        println!();
        println!("Hooks ({}):", config.hooks.len());
        for h in &config.hooks {
            println!("  • {} : {} {} ({}) → {}", h.hook_id, h.timing, h.transition_id, h.action, h.on_error);
        }
        println!();
        println!("Objets suivis: {}", engine.state_count());
        println!("Dry-run: {}", engine.dry_run());

        if engine.dry_run() {
            println!();
            println!("💡 Pour tester :");
            println!("  agent-manager -w {} -o projet-001 -t semer", cli.workflow);
        }
    }

    Ok(())
}
