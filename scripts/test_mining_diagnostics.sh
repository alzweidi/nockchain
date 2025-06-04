#!/bin/bash

echo "=== Mining Diagnostics ==="
echo "CPU Cores Available: $(nproc)"
echo "MINING_THREADS: ${MINING_THREADS:-not set}"
echo ""

# Check if rayon is detecting the thread pool
echo "=== Testing Rayon Thread Pool ==="
cat > test_rayon.rs << 'EOF'
use rayon::prelude::*;

fn main() {
    println!("Rayon threads: {}", rayon::current_num_threads());
    
    // Test parallel workload
    let sum: i32 = (0..1000000).into_par_iter().sum();
    println!("Parallel sum: {}", sum);
}
EOF

echo "Building rayon test..."
rustc test_rayon.rs --edition 2021 --extern rayon=$(find target -name "librayon*.rlib" | head -1) 2>/dev/null

if [ -f ./test_rayon ]; then
    ./test_rayon
    rm test_rayon test_rayon.rs
else
    echo "Could not test rayon directly"
fi

echo ""
echo "=== Checking Mining Process ==="

# Start mining with debug output
RUST_LOG=debug MINING_THREADS=16 timeout 10 cargo run --release -- --mine 2>&1 | grep -E "(worker|thread|parallel|Mining|FFT)" | head -20

echo ""
echo "=== Process Analysis ==="
echo "Running mining for 5 seconds to analyze CPU usage..."
MINING_THREADS=16 cargo run --release -- --mine &
MINING_PID=$!

sleep 2

# Check CPU usage
echo "Top processes:"
ps aux | grep -E "(nockchain|mining)" | grep -v grep

# Check thread count
if [ -f /proc/$MINING_PID/status ]; then
    echo ""
    echo "Thread count for PID $MINING_PID:"
    grep Threads /proc/$MINING_PID/status
fi

kill $MINING_PID 2>/dev/null

echo ""
echo "=== Possible Issues ==="
echo "1. Check if the polynomial sizes in STARK proofs are large enough to trigger parallel paths (>1024 elements)"
echo "2. The bottleneck might be in the Hoon kernel loading or single-threaded Hoon execution"
echo "3. Worker threads might be blocked waiting for kernel operations" 
