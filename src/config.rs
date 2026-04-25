// Lecture et validation des JSON de workflow

use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
pub struct WorkflowConfig {
    // TODO: Structurer selon workflow-json-specs-v2.md
}

impl WorkflowConfig {
    pub fn load(_dir: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        // TODO: Lire states.json, transitions.json, hooks.json, config.json
        log::info!("WorkflowConfig::load (placeholder)");
        Ok(Self {})
    }
}
