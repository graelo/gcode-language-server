use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use gcode_language_server::gcode::*;
use std::io::Cursor;

fn generate_gcode_content(size_mb: usize) -> String {
    let target_bytes = size_mb * 1024 * 1024;
    let mut content = String::with_capacity(target_bytes + 1000);

    // Generate realistic G-code patterns
    let patterns = [
        "G28 ; home all axes\n",
        "M104 S210 ; set hotend temperature\n",
        "M190 S60 ; wait for bed temperature\n",
        "G1 X10.0 Y10.0 Z0.3 F1500 ; move to position\n",
        "G1 X20.0 Y20.0 E0.1 F3000 ; extrude\n",
        "G1 X30.0 Y30.0 E0.2 ; continue extrusion\n",
        "; layer change\n",
        "G0 Z0.6 ; lift Z\n",
        "(temperature check)\n",
        "M105 ; report temperatures\n",
    ];

    let mut pattern_index = 0;
    while content.len() < target_bytes {
        content.push_str(patterns[pattern_index % patterns.len()]);
        pattern_index += 1;
    }

    content
}

fn bench_tokenize_text(c: &mut Criterion) {
    let mut group = c.benchmark_group("tokenize_text");

    for size_mb in [1, 5, 10, 20].iter() {
        let content = generate_gcode_content(*size_mb);
        group.throughput(Throughput::Bytes(content.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("size_mb", size_mb),
            &content,
            |b, content| {
                b.iter(|| {
                    let tokens = tokenize_text(content);
                    criterion::black_box(tokens.len())
                })
            },
        );
    }
    group.finish();
}

fn bench_streaming_iterator(c: &mut Criterion) {
    let mut group = c.benchmark_group("streaming_iterator");

    for size_mb in [1, 5, 10, 20].iter() {
        let content = generate_gcode_content(*size_mb);
        group.throughput(Throughput::Bytes(content.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("size_mb", size_mb),
            &content,
            |b, content| {
                b.iter(|| {
                    let cursor = Cursor::new(content.as_bytes());
                    let iterator = TokenIterator::new(cursor);
                    let count = iterator.count();
                    criterion::black_box(count)
                })
            },
        );
    }
    group.finish();
}

fn bench_token_at_lookup(c: &mut Criterion) {
    let content = generate_gcode_content(5); // 5MB for lookup tests
    let tokens = tokenize_text(&content);

    c.bench_function("token_at_lookup", |b| {
        b.iter(|| {
            // Test lookups at various positions
            for &pos in [1000, 50000, 100000, 500000].iter() {
                let token = token_at_position(&tokens, pos);
                criterion::black_box(token);
            }
        })
    });
}

criterion_group!(
    benches,
    bench_tokenize_text,
    bench_streaming_iterator,
    bench_token_at_lookup
);
criterion_main!(benches);
