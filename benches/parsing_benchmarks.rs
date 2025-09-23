use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use gcode_language_server::{parse_line, ParsedLine};
use std::fs;

/// Generate G-code content of different patterns for benchmarking
fn generate_gcode_content(lines: usize, pattern: &str) -> String {
    let mut content = String::new();

    match pattern {
        "movement_heavy" => {
            for i in 0..lines {
                content.push_str(&format!(
                    "G1 X{:.3} Y{:.3} Z{:.3} E{:.3} F1500\n",
                    (i as f32) * 0.1,
                    (i as f32) * 0.2,
                    (i as f32) * 0.05,
                    (i as f32) * 0.02
                ));
            }
        }
        "parameter_heavy" => {
            for i in 0..lines {
                content.push_str(&format!(
                    "M851 X{:.3} Y{:.3} Z{:.3} I{} J{} P{} S{}\n",
                    (i as f32) * 0.1,
                    (i as f32) * 0.1,
                    (i as f32) * 0.01,
                    i % 10,
                    i % 5,
                    i % 3,
                    i % 2
                ));
            }
        }
        "comment_heavy" => {
            for i in 0..lines {
                content.push_str(&format!(
                    "G1 X{:.1} Y{:.1} ; Move to position {}, layer {}, segment {}\n",
                    (i as f32) * 0.1,
                    (i as f32) * 0.1,
                    i,
                    i / 100,
                    i % 100
                ));
            }
        }
        "mixed" => {
            for i in 0..lines {
                match i % 4 {
                    0 => content.push_str(&format!(
                        "G1 X{:.3} Y{:.3} F1500\n",
                        (i as f32) * 0.1,
                        (i as f32) * 0.2
                    )),
                    1 => content.push_str(&format!("; Layer {}\n", i / 4)),
                    2 => content.push_str(&format!("M104 S{}\n", 200 + (i % 50))),
                    3 => content.push_str(&format!("G0 Z{:.2}\n", (i as f32) * 0.1)),
                    _ => unreachable!(),
                }
            }
        }
        _ => {
            for i in 0..lines {
                content.push_str(&format!("G1 X{} Y{}\n", i, i));
            }
        }
    }

    content
}

/// Benchmark parsing single lines with different patterns
fn bench_single_line_parsing(c: &mut Criterion) {
    let test_lines = vec![
        ("simple_move", "G1 X10 Y20"),
        ("complex_move", "G1 X123.456 Y789.012 Z0.3 E2.85714 F1500"),
        ("parameter_heavy", "M851 X1.23 Y-2.45 Z0.67 I1 J2 P3 S4 T5"),
        ("with_comment", "G1 X10 Y20 ; Move to next position"),
        (
            "comment_only",
            "; This is a comment line with some detailed information",
        ),
        ("temperature", "M104 S210 T0"),
        ("home_command", "G28 X Y Z"),
    ];

    let mut group = c.benchmark_group("single_line_parsing");

    for (name, line) in test_lines {
        group.bench_with_input(BenchmarkId::new("parse_line", name), &line, |b, line| {
            b.iter(|| black_box(parse_line(black_box(line))))
        });
    }

    group.finish();
}

/// Benchmark parsing files of different sizes
fn bench_file_parsing(c: &mut Criterion) {
    let file_sizes = vec![100, 1_000, 10_000, 100_000];
    let patterns = vec![
        "movement_heavy",
        "parameter_heavy",
        "comment_heavy",
        "mixed",
    ];

    let mut group = c.benchmark_group("file_parsing");

    for &size in &file_sizes {
        for pattern in &patterns {
            let content = generate_gcode_content(size, pattern);
            let lines: Vec<&str> = content.lines().collect();

            group.throughput(Throughput::Elements(size as u64));
            group.bench_with_input(
                BenchmarkId::new(format!("{}_{}", pattern, size), size),
                &lines,
                |b, lines| {
                    b.iter(|| {
                        let results: Vec<ParsedLine> = lines
                            .iter()
                            .map(|line| black_box(parse_line(black_box(line))))
                            .collect();
                        black_box(results)
                    })
                },
            );
        }
    }

    group.finish();
}

/// Benchmark parsing throughput (bytes per second)
fn bench_parsing_throughput(c: &mut Criterion) {
    let file_sizes = vec![1_000, 10_000, 100_000];

    let mut group = c.benchmark_group("parsing_throughput");

    for &size in &file_sizes {
        let content = generate_gcode_content(size, "mixed");
        let byte_size = content.len();

        group.throughput(Throughput::Bytes(byte_size as u64));
        group.bench_with_input(
            BenchmarkId::new("throughput", size),
            &content,
            |b, content| {
                b.iter(|| {
                    let results: Vec<ParsedLine> = content
                        .lines()
                        .map(|line| black_box(parse_line(black_box(line))))
                        .collect();
                    black_box(results)
                })
            },
        );
    }

    group.finish();
}

/// Benchmark parsing real G-code files from the fixtures
fn bench_real_files(c: &mut Criterion) {
    let fixture_files = vec![
        "tests/fixtures/sample_prusa.gcode",
        "tests/fixtures/sample_workspace.gcode",
        "tests/fixtures/sample_bottom_modeline.gcode",
    ];

    let mut group = c.benchmark_group("real_files");

    for file_path in fixture_files {
        if let Ok(content) = fs::read_to_string(file_path) {
            let lines: Vec<&str> = content.lines().collect();
            let file_name = file_path.split('/').last().unwrap_or("unknown");
            let byte_size = content.len();

            group.throughput(Throughput::Bytes(byte_size as u64));
            group.bench_with_input(
                BenchmarkId::new("real_file", file_name),
                &lines,
                |b, lines| {
                    b.iter(|| {
                        let results: Vec<ParsedLine> = lines
                            .iter()
                            .map(|line| black_box(parse_line(black_box(line))))
                            .collect();
                        black_box(results)
                    })
                },
            );
        }
    }

    group.finish();
}

/// Benchmark tokenization performance separately
fn bench_tokenization(c: &mut Criterion) {
    use gcode_language_server::parser::tokenize_line;

    let test_lines = vec![
        ("simple", "G1 X10 Y20"),
        (
            "complex",
            "G1 X123.456 Y789.012 Z0.3 E2.85714 F1500 ; Complex move",
        ),
        (
            "many_params",
            "M851 X1.23 Y-2.45 Z0.67 I1 J2 P3 S4 T5 U6 V7 W8",
        ),
    ];

    let mut group = c.benchmark_group("tokenization");

    for (name, line) in test_lines {
        group.bench_with_input(BenchmarkId::new("tokenize", name), &line, |b, line| {
            b.iter(|| black_box(tokenize_line(black_box(line))))
        });
    }

    group.finish();
}

criterion_group!(
    parsing_benches,
    bench_single_line_parsing,
    bench_file_parsing,
    bench_parsing_throughput,
    bench_real_files,
    bench_tokenization
);

criterion_main!(parsing_benches);
