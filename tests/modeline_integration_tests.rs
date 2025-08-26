//! Test modeline detection with actual file content and document states
use gcode_language_server::flavor::FlavorManager;

#[tokio::test]
async fn test_modeline_with_sample_files() {
    let manager = FlavorManager::with_default_config().expect("create manager");

    // Test Prusa modeline detection
    let prusa_content =
        std::fs::read_to_string("examples/sample_prusa.gcode").expect("read prusa sample file");

    let detected = manager.detect_modeline_flavor(&prusa_content);
    assert_eq!(detected, Some("prusa".to_string()));

    // Test workspace modeline detection
    let workspace_content = std::fs::read_to_string("examples/sample_workspace.gcode")
        .expect("read workspace sample file");

    let detected = manager.detect_modeline_flavor(&workspace_content);
    assert_eq!(detected, Some("workspace_test".to_string()));
}

#[tokio::test]
async fn test_end_to_end_document_state() {
    let mut flavor_manager = FlavorManager::with_default_config().expect("create manager");
    flavor_manager.initialize(None).await.expect("initialize");

    let prusa_content =
        std::fs::read_to_string("examples/sample_prusa.gcode").expect("read prusa sample file");

    // Simulate what happens when a document is opened
    let flavor_name = flavor_manager.detect_modeline_flavor(&prusa_content);
    assert_eq!(flavor_name, Some("prusa".to_string()));

    // Get the appropriate flavor
    let loaded_flavor = flavor_manager
        .get_flavor("prusa")
        .await
        .expect("get prusa flavor");
    let commands = flavor_manager.flavor_to_command_map(&loaded_flavor.flavor);

    // Should have M250 (Prusa-specific command)
    assert!(commands.contains_key("M250"));
    let m250 = commands.get("M250").unwrap();
    assert!(m250.description_short.is_some());

    // Should have standard G-code commands too
    assert!(commands.contains_key("G28"));
    assert!(commands.contains_key("G1"));
}
