use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use gcode_language_server::{validate_document, FlavorRegistry};

/// Generate test content with specific validation scenarios
fn generate_validation_content(lines: usize, scenario: &str) -> Vec<String> {
    let mut content = Vec::new();

    match scenario {
        "all_valid" => {
            for i in 0..lines {
                content.push(format!(
                    "G1 X{:.3} Y{:.3} F1500",
                    (i as f32) * 0.1,
                    (i as f32) * 0.1
                ));
            }
        }
        "missing_required_params" => {
            for i in 0..lines {
                if i % 3 == 0 {
                    content.push("G1".to_string()); // Missing required X,Y,Z parameters
                } else if i % 3 == 1 {
                    content.push("G0".to_string()); // Missing required coordinate parameters
                } else {
                    content.push(format!(
                        "G1 X{:.3} Y{:.3} F1500",
                        (i as f32) * 0.1,
                        (i as f32) * 0.1
                    ));
                }
            }
        }
        "unknown_commands" => {
            for i in 0..lines {
                if i % 4 == 0 {
                    content.push(format!("G999 X{}", i)); // Unknown G-code
                } else if i % 4 == 1 {
                    content.push(format!("M999 S{}", i)); // Unknown M-code
                } else if i % 4 == 2 {
                    content.push(format!("INVALID_CMD{}", i)); // Completely invalid
                } else {
                    content.push(format!(
                        "G1 X{:.3} Y{:.3} F1500",
                        (i as f32) * 0.1,
                        (i as f32) * 0.1
                    ));
                }
            }
        }
        "invalid_parameters" => {
            for i in 0..lines {
                if i % 5 == 0 {
                    content.push("G1 X Y10".to_string()); // Missing parameter value
                } else if i % 5 == 1 {
                    content.push("G1 Xinvalid Y10".to_string()); // Invalid parameter value
                } else if i % 5 == 2 {
                    content.push("G1 X10 Q15".to_string()); // Invalid parameter for command
                } else if i % 5 == 3 {
                    content.push("M104 S".to_string()); // Missing parameter value
                } else {
                    content.push(format!(
                        "G1 X{:.3} Y{:.3} F1500",
                        (i as f32) * 0.1,
                        (i as f32) * 0.1
                    ));
                }
            }
        }
        "complex_valid" => {
            for i in 0..lines {
                match i % 8 {
                    0 => content.push(format!(
                        "G1 X{:.3} Y{:.3} Z{:.3} E{:.5} F1500",
                        (i as f32) * 0.1,
                        (i as f32) * 0.1,
                        (i as f32) * 0.01,
                        (i as f32) * 0.02
                    )),
                    1 => content.push(format!("M104 S{}", 200 + (i % 50))),
                    2 => content.push(format!("M140 S{}", 60 + (i % 20))),
                    3 => content.push(format!(
                        "G0 X{:.2} Y{:.2} Z{:.2}",
                        (i as f32) * 0.1,
                        (i as f32) * 0.1,
                        (i as f32) * 0.1
                    )),
                    4 => content.push(format!("M106 S{}", i % 255)),
                    5 => content.push(format!(
                        "G2 X{:.2} Y{:.2} I{:.2} J{:.2}",
                        (i as f32) * 0.1,
                        (i as f32) * 0.1,
                        5.0,
                        5.0
                    )),
                    6 => content.push(format!("M851 Z{:.3}", (i as f32) * 0.001)),
                    7 => content.push("G28".to_string()),
                    _ => unreachable!(),
                }
            }
        }
        "mixed_errors" => {
            for i in 0..lines {
                match i % 10 {
                    0..=3 => content.push(format!(
                        "G1 X{:.3} Y{:.3} F1500",
                        (i as f32) * 0.1,
                        (i as f32) * 0.1
                    )), // Valid
                    4 => content.push("G1".to_string()), // Missing required params
                    5 => content.push("G999 X10".to_string()), // Unknown command
                    6 => content.push("G1 X Y10".to_string()), // Invalid parameter
                    7 => content.push("M104 S".to_string()), // Missing parameter value
                    8 => content.push("INVALID".to_string()), // Completely invalid
                    9 => content.push(format!("; Comment line {}", i)), // Comment (should be valid)
                    _ => unreachable!(),
                }
            }
        }
        _ => {
            for i in 0..lines {
                content.push(format!("G1 X{} Y{}", i, i));
            }
        }
    }

    content
}

/// Benchmark validation with different error densities
fn bench_validation_error_density(c: &mut Criterion) {
    let mut registry = FlavorRegistry::new();
    registry.add_embedded_prusa_flavor();
    registry.set_active_flavor("prusa");

    let scenarios = vec![
        ("all_valid", "All commands are valid"),
        ("missing_required_params", "33% missing required parameters"),
        ("unknown_commands", "75% unknown commands"),
        ("invalid_parameters", "80% invalid parameters"),
        ("complex_valid", "Complex but all valid commands"),
        ("mixed_errors", "40% various errors"),
    ];

    let mut group = c.benchmark_group("validation_error_density");

    for (scenario, _description) in scenarios {
        let content_lines = generate_validation_content(5000, scenario);
        let content = content_lines.join("\n");

        group.throughput(Throughput::Elements(content_lines.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("scenario", scenario),
            &content,
            |b, content| {
                b.iter(|| {
                    let result = validate_document(black_box(content), black_box(&registry));
                    black_box(result)
                })
            },
        );
    }

    group.finish();
}

/// Benchmark validation scalability with different file sizes
fn bench_validation_scalability(c: &mut Criterion) {
    let mut registry = FlavorRegistry::new();
    registry.add_embedded_prusa_flavor();

    let file_sizes = vec![100, 500, 1_000, 5_000, 10_000, 50_000];

    let mut group = c.benchmark_group("validation_scalability");

    for &size in &file_sizes {
        let content_lines = generate_validation_content(size, "mixed_errors");
        let content = content_lines.join("\n");
        let byte_size = content.len();

        group.throughput(Throughput::Bytes(byte_size as u64));
        group.bench_with_input(BenchmarkId::new("size", size), &content, |b, content| {
            b.iter(|| {
                let result = validate_document(black_box(content), black_box(&registry));
                black_box(result)
            })
        });
    }

    group.finish();
}

/// Benchmark validation of specific command types
fn bench_command_type_validation(c: &mut Criterion) {
    let mut registry = FlavorRegistry::new();
    registry.add_embedded_prusa_flavor();

    let command_scenarios = vec![
        (
            "movement_commands",
            vec![
                "G0 X10 Y20",
                "G1 X15 Y25 F1500",
                "G2 X20 Y30 I5 J5",
                "G3 X25 Y35 I-5 J-5",
            ],
        ),
        (
            "temperature_commands",
            vec!["M104 S210", "M109 S210", "M140 S60", "M190 S60"],
        ),
        (
            "fan_commands",
            vec!["M106 S255", "M106 S128", "M107", "M106 S0"],
        ),
        (
            "leveling_commands",
            vec!["G28", "G29", "M851 Z-0.1", "M420 S1"],
        ),
        ("extruder_commands", vec!["G10", "G11", "M82", "M83"]),
        (
            "misc_commands",
            vec!["M300 S1000 P200", "M117 Hello", "M42 P13 S1", "M280 P0 S90"],
        ),
    ];

    let mut group = c.benchmark_group("command_type_validation");

    for (scenario_name, commands) in command_scenarios {
        // Repeat each command set to create a larger test case
        let mut content_lines = Vec::new();
        for _ in 0..1000 {
            for cmd in &commands {
                content_lines.push(cmd.to_string());
            }
        }
        let content = content_lines.join("\n");

        group.throughput(Throughput::Elements(content_lines.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("command_type", scenario_name),
            &content,
            |b, content| {
                b.iter(|| {
                    let result = validate_document(black_box(content), black_box(&registry));
                    black_box(result)
                })
            },
        );
    }

    group.finish();
}

/// Benchmark flavor registry performance with different command lookup patterns
fn bench_flavor_registry_performance(c: &mut Criterion) {
    let mut registry = FlavorRegistry::new();
    registry.add_embedded_prusa_flavor();

    let mut group = c.benchmark_group("flavor_registry");

    // Common commands (should be fast to find)
    let common_commands = vec!["G0", "G1", "G28", "M104", "M140", "M106"];
    group.bench_function("common_commands", |b| {
        b.iter(|| {
            for cmd in &common_commands {
                let result = registry.get_command(black_box(cmd));
                black_box(result);
            }
        })
    });

    // Uncommon commands (might be slower to find)
    let uncommon_commands = vec!["G33", "M218", "M851", "M900", "M572", "M593"];
    group.bench_function("uncommon_commands", |b| {
        b.iter(|| {
            for cmd in &uncommon_commands {
                let result = registry.get_command(black_box(cmd));
                black_box(result);
            }
        })
    });

    // Non-existent commands (should be fast to reject)
    let invalid_commands = vec!["G999", "M999", "INVALID", "X123", "BADCMD", "G"];
    group.bench_function("invalid_commands", |b| {
        b.iter(|| {
            for cmd in &invalid_commands {
                let result = registry.get_command(black_box(cmd));
                black_box(result);
            }
        })
    });

    // Mixed command lookup pattern
    let mixed_commands = vec![
        "G1", "G999", "M104", "INVALID", "G28", "M999", "G0", "BADCMD",
    ];
    group.bench_function("mixed_commands", |b| {
        b.iter(|| {
            for cmd in &mixed_commands {
                let result = registry.get_command(black_box(cmd));
                black_box(result);
            }
        })
    });

    group.finish();
}

/// Benchmark parameter validation performance
fn bench_parameter_validation(c: &mut Criterion) {
    let mut registry = FlavorRegistry::new();
    registry.add_embedded_prusa_flavor();

    let parameter_scenarios = vec![
        (
            "valid_parameters",
            vec![
                "G1 X10.5 Y20.3 Z0.2 E1.5 F1500",
                "M104 S210 T0",
                "M140 S60",
                "G2 X10 Y10 I5 J5 F1000",
            ],
        ),
        (
            "missing_required",
            vec![
                "G1",         // Missing required coordinates
                "G0",         // Missing required coordinates
                "M104",       // Missing required temperature
                "G2 X10 Y10", // Missing required I,J parameters
            ],
        ),
        (
            "invalid_values",
            vec![
                "G1 X Y10",        // Missing X value
                "M104 Sinvalid",   // Invalid temperature value
                "G1 Xinvalid Y10", // Invalid coordinate value
                "M140 S-999",      // Out of range temperature
            ],
        ),
        (
            "extra_parameters",
            vec![
                "G1 X10 Y20 Q15", // Q parameter not valid for G1
                "M104 S210 X10",  // X parameter not valid for M104
                "G28 S100",       // S parameter not valid for G28
                "M140 S60 F1500", // F parameter not valid for M140
            ],
        ),
    ];

    let mut group = c.benchmark_group("parameter_validation");

    for (scenario_name, commands) in parameter_scenarios {
        // Create larger test cases by repeating commands
        let mut content_lines = Vec::new();
        for _ in 0..500 {
            for cmd in &commands {
                content_lines.push(cmd.to_string());
            }
        }
        let content = content_lines.join("\n");

        group.throughput(Throughput::Elements(content_lines.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("param_scenario", scenario_name),
            &content,
            |b, content| {
                b.iter(|| {
                    let result = validate_document(black_box(content), black_box(&registry));
                    black_box(result)
                })
            },
        );
    }

    group.finish();
}

/// Benchmark memory usage patterns (indirectly through performance)
fn bench_memory_patterns(c: &mut Criterion) {
    let mut registry = FlavorRegistry::new();
    registry.add_embedded_prusa_flavor();

    let mut group = c.benchmark_group("memory_patterns");

    // Large document with many errors (high memory pressure)
    let large_errors_lines = generate_validation_content(20_000, "mixed_errors");
    let large_errors = large_errors_lines.join("\n");

    group.bench_function("large_with_errors", |b| {
        b.iter(|| {
            let result = validate_document(black_box(&large_errors), black_box(&registry));
            black_box(result)
        })
    });

    // Large document with no errors (should use less memory for diagnostics)
    let large_clean_lines = generate_validation_content(20_000, "all_valid");
    let large_clean = large_clean_lines.join("\n");

    group.bench_function("large_clean", |b| {
        b.iter(|| {
            let result = validate_document(black_box(&large_clean), black_box(&registry));
            black_box(result)
        })
    });

    // Many small validations (simulating frequent LSP requests)
    let small_content_lines = generate_validation_content(100, "mixed_errors");
    let small_content = small_content_lines.join("\n");

    group.bench_function("frequent_small", |b| {
        b.iter(|| {
            // Simulate 100 small validation requests
            for _ in 0..100 {
                let result = validate_document(black_box(&small_content), black_box(&registry));
                black_box(result);
            }
        })
    });

    group.finish();
}

criterion_group!(
    validation_benches,
    bench_validation_error_density,
    bench_validation_scalability,
    bench_command_type_validation,
    bench_flavor_registry_performance,
    bench_parameter_validation,
    bench_memory_patterns
);

criterion_main!(validation_benches);
