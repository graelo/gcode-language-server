//! Test modeline detection at bottom of file
use gcode_language_server::flavor::FlavorManager;

#[tokio::test]
async fn test_modeline_at_bottom_of_file() {
    let manager = FlavorManager::with_default_config().expect("create manager");

    // Test file with modeline at the bottom
    let content = std::fs::read_to_string("tests/fixtures/sample_bottom_modeline.gcode")
        .expect("read sample file with bottom modeline");

    let detected = manager.detect_modeline_flavor(&content);
    assert_eq!(detected, Some("workspace_test".to_string()));

    println!(
        "✅ Successfully detected modeline at bottom: {:?}",
        detected
    );
}

#[test]
fn test_modeline_positions() {
    let manager = FlavorManager::with_default_config().expect("create manager");

    // Test modeline at top
    let content_top = r#"; gcode_flavor=marlin
G28
G1 X10
"#;
    assert_eq!(
        manager.detect_modeline_flavor(content_top),
        Some("marlin".to_string())
    );

    // Test modeline at bottom
    let content_bottom = r#"G28
G1 X10
G1 Y20
; more lines
; to make it longer
; gcode_flavor=prusa"#;
    assert_eq!(
        manager.detect_modeline_flavor(content_bottom),
        Some("prusa".to_string())
    );

    // Test modeline in middle of long file (should NOT be detected)
    let content_middle = format!(
        "{}\n; gcode_flavor=klipper\n{}",
        "G0\n".repeat(10), // 10 lines before
        "G1\n".repeat(10)  // 10 lines after
    );
    assert_eq!(manager.detect_modeline_flavor(&content_middle), None);

    println!("✅ All modeline position tests passed");
}
