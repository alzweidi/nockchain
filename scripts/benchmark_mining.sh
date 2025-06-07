#!/bin/bash
# Mining Performance Benchmark Script
# Usage: ./benchmark_mining.sh [threads] [duration_seconds]

THREADS=${1:-8}
DURATION=${2:-60}
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
LOG_FILE="mining_benchmark_${TIMESTAMP}.log"

echo "=== Nockchain Mining Benchmark ===" | tee $LOG_FILE
echo "Date: $(date)" | tee -a $LOG_FILE
echo "Threads: $THREADS" | tee -a $LOG_FILE
echo "Duration: $DURATION seconds" | tee -a $LOG_FILE
echo "=================================" | tee -a $LOG_FILE

# Check if nockchain binary exists
if ! command -v nockchain &> /dev/null; then
    echo "Building nockchain with optimized profile..."
    cargo build --profile mining --bin nockchain
    BINARY="./target/mining/nockchain"
else
    BINARY="nockchain"
fi

# Get system info (works on both Linux and macOS)
echo -e "\nSystem Information:" | tee -a $LOG_FILE
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    echo "CPU: $(sysctl -n machdep.cpu.brand_string)" | tee -a $LOG_FILE
    echo "Cores: $(sysctl -n hw.ncpu)" | tee -a $LOG_FILE
    echo "Memory: $(echo "scale=2; $(sysctl -n hw.memsize) / 1073741824" | bc) GB" | tee -a $LOG_FILE
else
    # Linux
    echo "CPU: $(lscpu | grep 'Model name' | cut -d':' -f2 | xargs)" | tee -a $LOG_FILE
    echo "Cores: $(nproc)" | tee -a $LOG_FILE
    echo "Memory: $(free -h | grep Mem | awk '{print $2}')" | tee -a $LOG_FILE
fi

# Create test configuration
TEST_KEY="test_mining_key_$(openssl rand -hex 16)"
echo -e "\nTest Configuration:" | tee -a $LOG_FILE
echo "Mining Key: $TEST_KEY" | tee -a $LOG_FILE

# Function to count proof attempts
count_proofs() {
    local log_file=$1
    grep -c "mining attempt" $log_file 2>/dev/null || echo 0
}

# Function to extract average proof time
avg_proof_time() {
    local log_file=$1
    grep "proof generation time" $log_file 2>/dev/null | \
        awk '{sum+=$NF; count++} END {if(count>0) print sum/count; else print 0}'
}

# Run mining benchmark
echo -e "\nStarting mining benchmark..." | tee -a $LOG_FILE
export MINING_THREADS=$THREADS
export RUST_LOG=info,nockchain::mining=debug

# Start mining in background with timeout
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS - use gtimeout if available, otherwise use a different approach
    if command -v gtimeout &> /dev/null; then
        gtimeout $DURATION $BINARY --mine --mining-key "$TEST_KEY" 2>&1 | tee mining_output_${TIMESTAMP}.log &
    else
        # Fallback for macOS without gtimeout
        $BINARY --mine --mining-key "$TEST_KEY" 2>&1 | tee mining_output_${TIMESTAMP}.log &
        MINING_PID=$!
        sleep $DURATION
        kill $MINING_PID 2>/dev/null
    fi
else
    # Linux
    timeout $DURATION $BINARY --mine --mining-key "$TEST_KEY" 2>&1 | tee mining_output_${TIMESTAMP}.log &
fi
MINING_PID=$!

# Monitor performance (simplified for cross-platform compatibility)
echo -e "\nMonitoring performance metrics..." | tee -a $LOG_FILE
for i in $(seq 1 $DURATION); do
    if [ $((i % 10)) -eq 0 ]; then
        echo "[$i s] Running..." | tee -a $LOG_FILE
    fi
    sleep 1
done

# Wait for process to finish
wait $MINING_PID 2>/dev/null

# Analyze results
echo -e "\n=== Benchmark Results ===" | tee -a $LOG_FILE
PROOF_COUNT=$(count_proofs mining_output_${TIMESTAMP}.log)
AVG_TIME=$(avg_proof_time mining_output_${TIMESTAMP}.log)

# Calculate proofs per second (handle division by zero)
if [ "$DURATION" -gt 0 ]; then
    PROOFS_PER_SEC=$(echo "scale=2; $PROOF_COUNT / $DURATION" | bc -l 2>/dev/null || echo "0")
else
    PROOFS_PER_SEC="0"
fi

echo "Total Proof Attempts: $PROOF_COUNT" | tee -a $LOG_FILE
echo "Average Proof Time: ${AVG_TIME}ms" | tee -a $LOG_FILE
echo "Proofs per Second: $PROOFS_PER_SEC" | tee -a $LOG_FILE

# Calculate efficiency
if [ "$THREADS" -gt 0 ] && [ "$PROOFS_PER_SEC" != "0" ]; then
    EFFICIENCY=$(echo "scale=2; $PROOFS_PER_SEC * 100 / $THREADS" | bc -l 2>/dev/null || echo "0")
    echo "Efficiency: ${EFFICIENCY}% per thread" | tee -a $LOG_FILE
fi

# Save detailed metrics
echo -e "\nDetailed metrics saved to:" | tee -a $LOG_FILE
echo "  - Summary: $LOG_FILE" | tee -a $LOG_FILE
echo "  - Full output: mining_output_${TIMESTAMP}.log" | tee -a $LOG_FILE

# Cleanup
echo -e "\nBenchmark complete!" | tee -a $LOG_FILE 
