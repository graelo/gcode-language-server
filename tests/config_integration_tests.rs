//! Tests for the configuration system
use gcode_language_server::Config;

#[test]
fn test_config_parsing() {
    // Test that config can be parsed without errors
    let config = Config::from_args_and_env().expect("create config");

    // Should have basic configuration
    assert!(!config.log_level.is_empty());

    // Should have flavor directories set up
    assert!(!config.flavor_dirs.is_empty());
}
