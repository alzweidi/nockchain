#!/bin/bash

echo "Testing Parallel Mining Configuration"
echo "====================================="

# Check CPU info
echo "System CPU Information:"
nproc
echo ""

# Test 1: Run with logging to see initialization
echo "Test 1: Checking thread pool initialization..."
echo "Running: RUST_LOG=info MINING_THREADS=32 timeout 10 nockchain --mining-pubkey test --mine"
RUST_LOG=info MINING_THREADS=32 timeout 10 nockchain --mining-pubkey test --mine 2>&1 | grep -E "(thread|Thread|worker|Worker|parallel|Parallel)" | head -20

echo ""
echo "Test 2: Monitoring CPU usage (run htop in another terminal)"
echo "Starting mining with 32 threads..."
echo ""

# Run the actual test
echo "Look for these in the logs:"
echo "- 'Initializing parallel mining thread pool with 32 threads'"
echo "- 'Mining configuration: Total threads: 32'"
echo "- 'All 8 mining workers spawned'"
echo "- 'bp_ntt_parallel called with n=X, using 32 threads'"
echo ""
echo "If mining has started, you should see:"
echo "- 'Received mining candidate #X'"
echo "- 'Worker X starting mining attempt'"
echo ""

# Run the jetpack tests to verify parallel operations work
echo "Test 3: Running parallel FFT correctness tests..."
cargo test -p zkvm-jetpack test_parallel_fft_correctness -- --nocapture
cargo test -p zkvm-jetpack test_parallel_multiplication -- --nocapture 
