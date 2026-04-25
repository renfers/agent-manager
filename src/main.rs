use std::path::PathBuf;
use clap::Parser;

mod engine;
mod registry;
mod config;
mod store;
mod actions;
mod objects;

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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let cli = Cli::parse();
    let _workflow_path = cli.registry.join(&cli.workflow);

    log::info!("agent-manager — workflow={} dry_run={}", cli.workflow, cli.dry_run);

    // Charger les 4 JSON (placeholder)
    let _config = config::WorkflowConfig::load(&_workflow_path)?;

    // Initialiser le moteur
    let mut engine = engine::WorkflowEngine::new();
    engine.run();

    log::info!("Arrêt du moteur");
    Ok(())
}
