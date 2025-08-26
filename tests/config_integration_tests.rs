//! Tests for the new configuration-based flavor selection
use gcode_language_server::{config::Config, flavor::FlavorManager};

#[tokio::test]
async fn test_project_config_flavor_selection() {
    // Test that project config is loaded and used for flavor selection
    let config = Config::from_args_and_env().expect("create config");
    
    // Should find the .gcode.toml in the project root
    assert!(config.has_project_config());
    assert_eq!(config.project_flavor.as_deref(), Some("prusa"));
    
    let effective = config.get_effective_flavor();
    assert_eq!(effective.as_deref(), Some("prusa"));
}

#[tokio::test] 
async fn test_flavor_manager_with_config() {
    let config = Config::from_args_and_env().expect("create config");
    let mut manager = FlavorManager::new(&config).expect("create manager");
    manager.initialize(None).await.expect("initialize");
    
    // Should use the effective default flavor based on config
    let effective_default = manager.get_effective_default_flavor().await;
    assert!(effective_default.is_some());
    
    // Should be the one specified in .gcode.toml (prusa)
    let flavor = effective_default.unwrap();
    assert_eq!(flavor.flavor.flavor.name, "prusa");
}

#[tokio::test]
async fn test_modeline_overrides_config() {
    let config = Config::from_args_and_env().expect("create config");
    let mut manager = FlavorManager::new(&config).expect("create manager");
    manager.initialize(None).await.expect("initialize");
    
    // Content with modeline should override project config
    let content_with_modeline = "; gcode_flavor=workspace_test\nG0 X10 Y10";
    
    // Modeline detection should work
    let detected = manager.detect_modeline_flavor(content_with_modeline);
    assert_eq!(detected, Some("workspace_test".to_string()));
    
    // When modeline is present, it should take precedence over config
    if let Some(workspace_flavor) = manager.get_flavor("workspace_test").await {
        assert_eq!(workspace_flavor.flavor.flavor.name, "workspace_test");
    }
}
