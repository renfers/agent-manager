use agent_manager::registry::{Registry, NativeAction, ScriptWrapper};

#[test]
fn test_registry_register_and_get() {
    let mut reg = Registry::new();
    let action = NativeAction { name: "test".into() };
    reg.register(Box::new(action));
    assert!(reg.get("test").is_some());
    assert!(reg.get("missing").is_none());
}

#[test]
fn test_registry_mixed_native_and_wrapper() {
    let mut reg = Registry::new();

    reg.register(Box::new(NativeAction { name: "send_telegram".into() }));
    reg.register(Box::new(ScriptWrapper {
        name: "call_hermes".into(),
        script_path: std::path::PathBuf::from("wrappers/call_hermes.py"),
        interpreter: "python3".into(),
    }));

    assert_eq!(reg.get("send_telegram").unwrap().name(), "send_telegram");
    assert_eq!(reg.get("call_hermes").unwrap().name(), "call_hermes");
}

#[tokio::test]
async fn test_workflow_load_config() {
    // TODO: Tester le chargement des 4 JSON depuis un dossier temporaire
}
