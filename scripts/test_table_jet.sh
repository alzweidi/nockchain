#!/bin/bash

echo "Testing build_table_dats_jet with debug output..."
echo "======================================"

# Set environment variables for debugging
export RUST_LOG=debug
export MINING_THREADS=1
export RUST_BACKTRACE=1

# Clean and rebuild
echo "Rebuilding with latest changes..."
cargo build --release --bin nockchain

# Run with mining and filter for relevant output
echo -e "\nRunning nockchain with mining enabled..."
echo "Watching for jet execution and errors..."
echo "======================================"

timeout 60 cargo run --release --bin nockchain -- --mine 2>&1 | tee debug_output.log | grep -E "(build_table_dats_jet|panic|thread|DIRECT_MAX|serf)"

echo -e "\n======================================"
echo "Debug output saved to debug_output.log"
echo "Checking for specific issues..."

# Check if the jet was called
if grep -q "build_table_dats_jet: Called" debug_output.log; then
    echo "✓ Jet was successfully called"
else
    echo "✗ Jet was not called"
fi

# Check for panics
if grep -q "panic" debug_output.log; then
    echo "✗ Panic detected - checking details..."
    grep -A5 -B5 "panic" debug_output.log
else
    echo "✓ No panics detected"
fi

# Check for DIRECT_MAX errors
if grep -q "DIRECT_MAX" debug_output.log; then
    echo "✗ DIRECT_MAX error detected"
else
    echo "✓ No DIRECT_MAX errors"
fi

echo -e "\nFull jet output:"
grep "build_table_dats_jet" debug_output.log || echo "No jet output found" 
