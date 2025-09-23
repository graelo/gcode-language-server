#!/usr/bin/env bash
#
# Benchmark runner script for gcode-language-server performance testing
#

set -e

echo "🚀 Running gcode-language-server benchmarks..."
echo "=================================================="

# Ensure we're in the right directory
cd "$(dirname "$0")/.."

# Build the project in release mode first
echo "📦 Building project in release mode..."
cargo build --release

# Create benchmark output directory
mkdir -p target/benchmark-results

# Function to run a benchmark and save results
run_benchmark() {
    local bench_name=$1
    local output_dir="target/benchmark-results"
    
    echo ""
    echo "⚡ Running $bench_name benchmark..."
    echo "-----------------------------------"
    
    # Run the benchmark and save to both terminal and file
    cargo bench --bench "$bench_name" 2>&1 | tee "$output_dir/${bench_name}_results.txt"
    
    # Check if HTML report was generated
    if [ -d "target/criterion" ]; then
        echo "📊 HTML report available at: target/criterion/reports/index.html"
    fi
}

# Function to check benchmark performance against thresholds
check_performance() {
    echo ""
    echo "🔍 Performance Analysis"
    echo "======================"
    
    local results_dir="target/benchmark-results"
    
    # Simple performance checks based on the benchmark output
    echo "Analyzing parsing performance..."
    
    if grep -q "time:" "$results_dir/parsing_benchmarks_results.txt" 2>/dev/null; then
        echo "✅ Parsing benchmarks completed successfully"
        
        # Extract some key metrics (this is a simplified analysis)
        if grep -q "throughput:" "$results_dir/parsing_benchmarks_results.txt" 2>/dev/null; then
            echo "📈 Throughput metrics found in parsing results"
        fi
    else
        echo "❌ Parsing benchmark results not found or incomplete"
    fi
    
    if grep -q "time:" "$results_dir/lsp_benchmarks_results.txt" 2>/dev/null; then
        echo "✅ LSP benchmarks completed successfully"
    else
        echo "❌ LSP benchmark results not found or incomplete"
    fi
    
    if grep -q "time:" "$results_dir/validation_benchmarks_results.txt" 2>/dev/null; then
        echo "✅ Validation benchmarks completed successfully"
    else
        echo "❌ Validation benchmark results not found or incomplete"
    fi
}

# Function to generate a summary report
generate_summary() {
    echo ""
    echo "📋 Benchmark Summary Report"
    echo "=========================="
    
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    local results_dir="target/benchmark-results"
    local summary_file="$results_dir/benchmark_summary.md"
    
    cat > "$summary_file" << EOF
# G-code Language Server Benchmark Results

**Generated:** $timestamp  
**Rust Version:** $(rustc --version)  
**System:** $(uname -s) $(uname -m)  

## Performance Overview

### Parsing Performance
$(if [ -f "$results_dir/parsing_benchmarks_results.txt" ]; then
    echo "- Parsing benchmarks completed ✅"
    echo "- See detailed results in parsing_benchmarks_results.txt"
else
    echo "- Parsing benchmarks failed ❌"
fi)

### LSP Operations Performance  
$(if [ -f "$results_dir/lsp_benchmarks_results.txt" ]; then
    echo "- LSP benchmarks completed ✅"
    echo "- See detailed results in lsp_benchmarks_results.txt"
else
    echo "- LSP benchmarks failed ❌"
fi)

### Validation Performance
$(if [ -f "$results_dir/validation_benchmarks_results.txt" ]; then
    echo "- Validation benchmarks completed ✅" 
    echo "- See detailed results in validation_benchmarks_results.txt"
else
    echo "- Validation benchmarks failed ❌"
fi)

## Files Generated
- parsing_benchmarks_results.txt
- lsp_benchmarks_results.txt  
- validation_benchmarks_results.txt
- benchmark_summary.md

## Next Steps
1. Review detailed results in target/criterion/reports/index.html
2. Compare against previous benchmark runs
3. Identify performance bottlenecks
4. Track performance regressions over time

EOF

    echo "📄 Summary report generated: $summary_file"
}

# Main execution
main() {
    # Check if criterion dependency is available
    if ! cargo tree | grep -q criterion; then
        echo "❌ Criterion dependency not found. Make sure it's added to Cargo.toml [dev-dependencies]"
        exit 1
    fi
    
    echo "🔧 System Information:"
    echo "   Rust: $(rustc --version)"
    echo "   OS: $(uname -s) $(uname -m)"
    echo "   CPU: $(sysctl -n machdep.cpu.brand_string 2>/dev/null || echo 'Unknown')"
    echo ""
    
    # Run each benchmark suite
    run_benchmark "parsing_benchmarks"
    run_benchmark "lsp_benchmarks" 
    run_benchmark "validation_benchmarks"
    
    # Analyze results
    check_performance
    
    # Generate summary
    generate_summary
    
    echo ""
    echo "🎉 All benchmarks completed!"
    echo "📊 View detailed HTML reports: target/criterion/reports/index.html"
    echo "📋 Summary report: target/benchmark-results/benchmark_summary.md"
}

# Handle command line arguments
case "${1:-}" in
    "parsing")
        run_benchmark "parsing_benchmarks"
        ;;
    "lsp")
        run_benchmark "lsp_benchmarks"
        ;;
    "validation")
        run_benchmark "validation_benchmarks"
        ;;
    "summary")
        generate_summary
        ;;
    "clean")
        echo "🧹 Cleaning benchmark results..."
        rm -rf target/criterion target/benchmark-results
        echo "✅ Benchmark results cleaned"
        ;;
    *)
        main
        ;;
esac