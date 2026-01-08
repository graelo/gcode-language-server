//! Tests for the configuration system
use clap::Parser;
use gcode_language_server::{Args, Config};

#[test]
fn test_config_parsing() {
    // Parse with just the program name (no extra args that could conflict with test runner)
    let args = Args::parse_from(["gcode-ls"]);
    let config = Config::from_args(args).expect("create config");

    // Should have basic configuration
    assert!(!config.log_level.is_empty());

    // Should have flavor directories set up (at least the user config dir)
    assert!(!config.flavor_dirs.is_empty());
}
