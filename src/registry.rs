// Registre d'actions — native et wrapper

use std::collections::HashMap;

pub trait ActionRegistry: Send + Sync {
    fn name(&self) -> &str;
    // TODO: fn handle(...)
    fn capabilities(&self) -> Vec<String> {
        vec![]
    }
}

pub struct NativeAction {
    pub name: String,
}

impl ActionRegistry for NativeAction {
    fn name(&self) -> &str {
        &self.name
    }
}

pub struct ScriptWrapper {
    pub name: String,
    pub script_path: std::path::PathBuf,
    pub interpreter: String,
}

impl ActionRegistry for ScriptWrapper {
    fn name(&self) -> &str {
        &self.name
    }
}

pub struct Registry {
    actions: HashMap<String, Box<dyn ActionRegistry>>,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            actions: HashMap::new(),
        }
    }

    pub fn register(&mut self, action: Box<dyn ActionRegistry>) {
        self.actions.insert(action.name().to_string(), action);
    }

    pub fn get(&self, name: &str) -> Option<&dyn ActionRegistry> {
        self.actions.get(name).map(|b| b.as_ref())
    }
}
