use gcode_language_server::parser::parse_line;

fn main() {
    println!("=== New Clean Parser Demo ===");

    let test_lines = [
        "G1 X10 Y20.5 Z0.2 ; linear move",
        "M104 S200 ; set temperature",
        "(this is a comment)",
        "; another comment",
        "",
        "G28 ; home all axes",
    ];

    for line in test_lines {
        println!("\nInput: '{}'", line);
        let result = parse_line(line);
        println!("Parsed: {:?}", result);
    }
}
