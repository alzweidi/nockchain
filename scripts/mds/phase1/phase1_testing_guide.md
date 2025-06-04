# Phase 1 Table Jet Testing Guide

## Overview

This guide explains how to test the Phase 1 table building jet implementation and verify it's working correctly.

## What We've Implemented

1. **`build_table_dats_jet`** in `crates/zkvm-jetpack/src/jets/table_jets.rs`
   - Jets the Hoon function at `hoon/common/stark/prover.hoon:551`
   - Builds compute and memory tables from the execution trace
   - Currently a simplified implementation that demonstrates the approach

2. **Integration** in `crates/zkvm-jetpack/src/hot.rs`
   - Added to the prover hot state
   - Registered with the correct jet path

## How to Test

### 1. Build the Project

```bash
cargo build --release
```

### 2. Run with Debug Output

To see the jet being called:

```bash
RUST_LOG=debug MINING_THREADS=1 cargo run --release --bin nockchain -- --mine 2>&1 | grep "build_table_dats_jet"
```

You should see output like:
```
build_table_dats_jet: Called! This jet is working.
build_table_dats_jet: Processing fock-return data
build_table_dats_jet: Building table 'memory'
build_table_dats_jet: Building table 'compute'
build_table_dats_jet: Successfully built 2 tables
```

### 3. Performance Comparison

To measure the speedup:

#### Baseline (without jet):
1. Comment out the jet in `hot.rs` (remove `jets.extend(TABLE_JETS);`)
2. Time the mining:
```bash
time timeout 60s cargo run --release --bin nockchain -- --mine
```

#### With Jet:
1. Ensure the jet is enabled in `hot.rs`
2. Time the mining:
```bash
time timeout 60s cargo run --release --bin nockchain -- --mine
```

### 4. Correctness Verification

The jet should produce identical results to the Hoon implementation. To verify:

1. Add more detailed logging to compare table structures
2. Run tests that compare jet output with Hoon output
3. Ensure proofs still verify correctly

## Current Limitations

1. **Simplified Implementation**: The current jet is a proof of concept that:
   - Only builds basic table structure
   - Doesn't include all table operations
   - Returns placeholder values for table-funcs and verifier-funcs

2. **No Parallelization Yet**: The implementation is sequential for now

## Next Steps

1. **Complete Implementation**:
   - Process all queue operations correctly
   - Build proper memory table
   - Include actual table-funcs and verifier-funcs

2. **Add Parallelization**:
   - Use rayon to parallelize row processing
   - Parallel table building for compute and memory

3. **Comprehensive Testing**:
   - Unit tests comparing with Hoon output
   - Performance benchmarks
   - Integration tests with full proof generation

## Expected Results

Even this simplified implementation should show:
- Jet is being called during proof generation
- Some speedup from removing interpreter overhead
- Correct basic table structure

Full implementation should achieve:
- 2-3x overall speedup in proof generation
- Significant reduction in table building time
- Full compatibility with existing proof verification

## Debugging Tips

1. **Jet Not Called**: 
   - Ensure the jet path matches exactly
   - Check that hot state includes TABLE_JETS
   - Verify the Hoon code has the jet hint

2. **Incorrect Output**:
   - Add more debug logging
   - Compare step-by-step with Hoon implementation
   - Check noun structure matches expected format

3. **Performance Issues**:
   - Profile with `perf` or `flamegraph`
   - Check for unnecessary allocations
   - Ensure efficient data structure usage 
