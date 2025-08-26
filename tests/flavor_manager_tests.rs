//! Tests specifically for the flavor manager functionality
use gcode_language_server::flavor::{FlavorManager, FlavorPriority};

#[tokio::test]
async fn test_flavor_loading_priority() {
    let mut manager = FlavorManager::with_default_config().expect("create manager");
    manager.initialize(None).await.expect("initialize");

    // Should load built-in Prusa flavor
    let prusa = manager.get_flavor("prusa").await;
    assert!(prusa.is_some());
    assert_eq!(prusa.unwrap().priority, FlavorPriority::BuiltIn);
}

#[tokio::test]
async fn test_flavor_commands_map() {
    let mut manager = FlavorManager::with_default_config().expect("create manager");
    manager.initialize(None).await.expect("initialize");

    let prusa = manager.get_flavor("prusa").await.expect("prusa flavor");
    let commands = manager.flavor_to_command_map(&prusa.flavor);

    // Should have basic G-code commands
    assert!(commands.contains_key("G0"));
    assert!(commands.contains_key("G1"));
    assert!(commands.contains_key("G28"));
    assert!(commands.contains_key("M250"));

    // Check command details
    let g1_cmd = commands.get("G1").expect("G1 command");
    assert_eq!(g1_cmd.name, "G1");
    assert!(g1_cmd.description_short.is_some());
}

#[tokio::test]
async fn test_modeline_detection() {
    let manager = FlavorManager::with_default_config().expect("create manager");

    // Test various modeline formats
    let content1 = "; vim: gcode_flavor=prusa\nG1 X10 Y20";
    assert_eq!(
        manager.detect_modeline_flavor(content1),
        Some("prusa".to_string())
    );

    let content2 = "; gcode_flavor=custom_flavor\nG0 X0";
    assert_eq!(
        manager.detect_modeline_flavor(content2),
        Some("custom_flavor".to_string())
    );

    let content3 = "// gcode_flavor=reprap\nG28";
    assert_eq!(
        manager.detect_modeline_flavor(content3),
        Some("reprap".to_string())
    );

    // Test content without modeline
    let content4 = "G1 X10 Y20\n; just a comment";
    assert_eq!(manager.detect_modeline_flavor(content4), None);
}

#[tokio::test]
async fn test_default_flavor_selection() {
    let mut manager = FlavorManager::with_default_config().expect("create manager");
    manager.initialize(None).await.expect("initialize");

    let default = manager.get_default_flavor().await;
    assert!(default.is_some());

    // Should be Prusa since that's our built-in
    let default_flavor = default.unwrap();
    assert_eq!(default_flavor.flavor.flavor.name, "prusa");
}

#[tokio::test]
async fn test_list_flavor_names() {
    let mut manager = FlavorManager::with_default_config().expect("create manager");
    manager.initialize(None).await.expect("initialize");

    let names = manager.list_flavor_names().await;
    assert!(names.contains(&"prusa".to_string()));
    assert!(!names.is_empty());
}
