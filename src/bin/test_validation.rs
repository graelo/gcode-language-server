use gcode_language_server::validation::engine::{validate_document, Severity};
use gcode_language_server::flavor::registry::FlavorRegistry;

fn main() {
    // Test validation with G0/G1 with and without parameters
    let test_content = r#"; Test cases for G0/G1 validation
G0
G1
G0 X10
G1 X10 Y20
G0 F1800
G1 E5.0 F1800
"#;

    let mut registry = FlavorRegistry::new();
    registry.add_embedded_prusa_flavor();
    assert!(registry.set_active_flavor("prusa"));

    let result = validate_document(test_content, &registry);
    
    println!("Validation result:");
    println!("Total diagnostics: {}", result.diagnostics.len());
    
    for diagnostic in &result.diagnostics {
        println!("Line {}: {:?} - {}", diagnostic.line, diagnostic.severity, diagnostic.message);
    }
    
    // Check that G0 and G1 without coordinates produce errors
    let g0_errors: Vec<_> = result.diagnostics.iter()
        .filter(|d| d.line == 2 && d.severity == Severity::Error && d.message.contains("G0"))
        .collect();
    let g1_errors: Vec<_> = result.diagnostics.iter()
        .filter(|d| d.line == 3 && d.severity == Severity::Error && d.message.contains("G1"))
        .collect();
        
    println!("\nG0 errors (line 2): {}", g0_errors.len());
    println!("G1 errors (line 3): {}", g1_errors.len());
    
    // Check that G0/G1 with only non-coordinate parameters produce errors  
    let g0_f_errors: Vec<_> = result.diagnostics.iter()
        .filter(|d| d.line == 6 && d.severity == Severity::Error && d.message.contains("G0"))
        .collect();
    let g1_ef_errors: Vec<_> = result.diagnostics.iter()
        .filter(|d| d.line == 7 && d.severity == Severity::Error && d.message.contains("G1"))
        .collect();
        
    println!("G0 with F only errors (line 6): {}", g0_f_errors.len());
    println!("G1 with E+F only errors (line 7): {}", g1_ef_errors.len());
    
    // Verify no errors for valid coordinate commands
    let coord_errors: Vec<_> = result.diagnostics.iter()
        .filter(|d| (d.line == 4 || d.line == 5) && d.severity == Severity::Error)
        .collect();
    println!("Coordinate command errors (lines 4-5): {}", coord_errors.len());
}