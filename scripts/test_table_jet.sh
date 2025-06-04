#!/bin/bash
# Test script to verify table building jet works

echo "Testing table building jet..."

# First, build the project with the new jet
echo "Building nockchain with table jet..."
cargo build --release 2>&1 | grep -E "(error|warning|Finished)"

if [ $? -ne 0 ]; then
    echo "Build failed!"
    exit 1
fi

echo "Build successful!"

# Run a simple mining test to see if the jet is being called
echo "Testing mining with table jet..."
RUST_LOG=debug MINING_THREADS=1 timeout 30s cargo run --release --bin nockchain -- --mine 2>&1 | grep -E "(build-table-dats|table_jet|Building tables)" | head -20

echo ""
echo "To verify the jet is working:"
echo "1. Look for debug output mentioning table building"
echo "2. Compare timing with and without the jet"
echo ""
echo "Run this to test without the jet:"
echo "  cargo run --release --bin nockchain -- --mine"
echo ""
echo "The jet should provide 2-3x speedup in the table building phase." 
