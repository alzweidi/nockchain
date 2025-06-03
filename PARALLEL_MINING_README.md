# Parallel Mining Testing Instructions

## Quick Start

### Basic Mining Test
```bash
# Run with default settings (uses all CPU cores)
nockchain --mine

# Run with specific thread count
MINING_THREADS=8 nockchain --mine

# Run with optimal thread count for your system
MINING_THREADS=$(nproc) nockchain --mine
```

## Performance Testing

### 1. Baseline Test (Single Thread)
```bash
# Force single-threaded operation for comparison
MINING_THREADS=1 nockchain --mine
```
Note the time taken to find a valid proof.

### 2. Parallel Test (Multi-threaded)
```bash
# Use all available cores
nockchain --mine
```
Compare the time taken with the single-threaded baseline.

### 3. Different Thread Configurations
```bash
# Test with different thread counts
for threads in 1 2 4 8 16; do
    echo "Testing with $threads threads..."
    MINING_THREADS=$threads timeout 300 nockchain --mine
done
```

## Monitoring Performance

### Watch CPU Usage
```bash
# In another terminal, monitor CPU usage
htop
# or
top -H
```
You should see multiple `mining-worker-*` threads utilizing CPU cores.

### Check Mining Logs
The logs will show:
```
Mining configuration:
  Total threads: 16
  Worker count: 4
  Threads per worker: 4
```

And worker activity:
```
Mining worker 0 initialized successfully
Worker 0 completed mining attempt in 1.2s
Worker 1 completed mining attempt in 1.3s
```

## Configuration Examples

### Laptop (4 cores)
```bash
MINING_THREADS=4 nockchain --mine
# Creates 1 worker using all 4 threads for maximum speed per proof
```

### Desktop (8 cores)
```bash
MINING_THREADS=8 nockchain --mine
# Creates 2 workers, each using 4 threads
```

### Server (32 cores)
```bash
MINING_THREADS=32 nockchain --mine
# Creates 8 workers, each using 4 threads
```

## Verification Tests

### 1. Correctness Test
```bash
# Run with logging to verify proofs are valid
RUST_LOG=info MINING_THREADS=8 nockchain --mine
```

### 2. Stress Test
```bash
# Run for extended period to ensure stability
MINING_THREADS=$(nproc) timeout 3600 nockchain --mine
```

### 3. Memory Usage Test
```bash
# Monitor memory usage while mining
MINING_THREADS=16 nockchain --mine &
PID=$!
while kill -0 $PID 2>/dev/null; do
    ps -p $PID -o pid,vsz,rss,comm
    sleep 5
done
```

## Expected Performance Gains

### Compared to Original Implementation
- **4 cores**: ~3-4x faster
- **8 cores**: ~6-7x faster
- **16 cores**: ~10-12x faster
- **32 cores**: ~15-20x faster

### Performance Metrics to Track
1. **Time to find valid proof** - Should decrease with more threads
2. **CPU utilization** - Should approach 100% on all cores
3. **Memory usage** - Should remain stable (not growing over time)
4. **Proofs per minute** - Should scale with thread count

## Troubleshooting

### Low CPU Usage
```bash
# Check thread configuration
RUST_LOG=info MINING_THREADS=16 nockchain --mine
```

### Comparing with Original
```bash
# If you have the original binary
time ./nockchain-original --mine
time MINING_THREADS=$(nproc) ./nockchain --mine
```

## Advanced Testing

### Profile Performance
```bash
# Generate flamegraph (requires cargo-flamegraph)
MINING_THREADS=8 cargo flamegraph --bin nockchain -- --mine
```

### Benchmark Specific Components
```bash
# Run jetpack tests to verify parallel FFT correctness
cargo test -p zkvm-jetpack test_parallel_fft_correctness
cargo test -p zkvm-jetpack test_parallel_multiplication
```

## Integration Testing

### With Mining Pool
```bash
# Ensure compatibility with mining pool protocols
MINING_THREADS=16 nockchain --mine --pool-url <your-pool>
```

### Long-Running Test
```bash
# Run overnight to ensure no memory leaks or crashes
MINING_THREADS=$(nproc) nohup nockchain --mine > mining.log 2>&1 &
```

## Reporting Results

When reporting performance, include:
1. System specs (CPU model, core count, RAM)
2. Thread configuration used
3. Average time to find proof
4. Comparison with single-threaded performance
5. Any errors or warnings observed

Example:
```
System: AMD Ryzen 9 5950X (16 cores, 32 threads)
Config: MINING_THREADS=16 (4 workers × 4 threads)
Results: 
  - Single-threaded: 10.2s average
  - Parallel (16 threads): 0.8s average
  - Speedup: 12.75x
  - CPU usage: 95-100%
  - Memory: Stable at 1.2GB
``` 
