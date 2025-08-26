//! Integration tests for file watching and workspace flavor loading
use gcode_language_server::flavor::FlavorManager;
use std::time::Duration;
use tokio::fs;

#[tokio::test]
async fn test_workspace_flavor_loading() {
    let mut manager = FlavorManager::with_default_config().expect("create manager");
    manager.initialize(None).await.expect("initialize");

    // Should load workspace flavor
    let workspace_flavor = manager.get_flavor("workspace_test").await;
    assert!(workspace_flavor.is_some());

    let flavor = workspace_flavor.unwrap();
    let commands = manager.flavor_to_command_map(&flavor.flavor);

    // Should have the custom command
    assert!(commands.contains_key("G99"));

    // Should have overridden G0
    let g0_cmd = commands.get("G0").expect("G0 command");
    assert_eq!(
        g0_cmd.description_short,
        Some("Workspace rapid move".to_string())
    );
}

#[tokio::test]
async fn test_flavor_priority() {
    let mut manager = FlavorManager::with_default_config().expect("create manager");
    manager.initialize(None).await.expect("initialize");

    // Get the built-in prusa flavor
    let prusa = manager.get_flavor("prusa").await.expect("prusa flavor");

    // Get the workspace test flavor
    let workspace = manager
        .get_flavor("workspace_test")
        .await
        .expect("workspace flavor");

    // Workspace should have higher priority than built-in
    assert!(workspace.priority > prusa.priority);
}

#[tokio::test]
async fn test_flavor_file_modification() {
    let mut manager = FlavorManager::with_default_config().expect("create manager");
    manager.initialize(None).await.expect("initialize");

    let flavor_path = ".gcode-ls/flavors/test_dynamic.toml";

    // Create a new flavor file
    let initial_content = r#"
[flavor]
name = "test_dynamic"
version = "1.0"

[[commands]]
name = "G100"
description_short = "Initial description"
"#;

    fs::write(flavor_path, initial_content)
        .await
        .expect("write initial file");

    // Wait for file watcher to detect the change
    tokio::time::sleep(Duration::from_millis(1500)).await;

    // The flavor should be available now (this tests file watching)
    let flavor = manager.get_flavor("test_dynamic").await;
    assert!(flavor.is_some(), "Dynamic flavor should be loaded");

    // Clean up
    let _ = fs::remove_file(flavor_path).await;
}
