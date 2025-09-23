use gcode_language_server::validation::engine::{validate_document, Severity};
use gcode_language_server::flavor::registry::FlavorRegistry;
use gcode_language_server::flavor::schema::{Flavor, FlavorFile};
use std::env;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    // Parse command line arguments
    let mut flavor_name = "prusa".to_string();
    let mut file_content = None;
    
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--flavor" => {
                if i + 1 < args.len() {
                    flavor_name = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("Error: --flavor requires a value");
                    std::process::exit(1);
                }
            }
            arg if !arg.starts_with("--") => {
                // Treat as filename
                file_content = Some(fs::read_to_string(arg)?);
                i += 1;
            }
            _ => {
                eprintln!("Usage: test_validation [--flavor <flavor>] [filename]");
                std::process::exit(1);
            }
        }
    }
    
    // Use provided file content or default test content
    let test_content = file_content.unwrap_or_else(|| {
        r#"; Test cases for G0/G1 validation
G0
G1
G0 X10
G1 X10 Y20
G0 F1800
G1 E5.0 F1800
"#.to_string()
    });

    let mut registry = FlavorRegistry::new();
    
    // Load the specified flavor
    match flavor_name.as_str() {
        "prusa" => {
            registry.add_embedded_prusa_flavor();
        }
        "marlin" => {
            let marlin_toml = fs::read_to_string("resources/flavors/marlin.gcode-flavor.toml")?;
            let flavor_file: FlavorFile = toml::from_str(&marlin_toml)?;
            let flavor = Flavor::from(flavor_file);
            registry.add_flavor(flavor);
        }
        _ => {
            eprintln!("Error: Unknown flavor '{}'", flavor_name);
            std::process::exit(1);
        }
    }
    
    if !registry.set_active_flavor(&flavor_name) {
        eprintln!("Error: Failed to activate flavor '{}'", flavor_name);
        std::process::exit(1);
    }

    let result = validate_document(&test_content, &registry);
    
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
    
    Ok(())
}