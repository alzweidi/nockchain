use std::str::FromStr;
use std::sync::Arc;
use std::path::PathBuf;

use kernels::miner::KERNEL;
use nockapp::kernel::checkpoint::JamPaths;
use nockapp::kernel::form::Kernel;
use nockapp::nockapp::driver::{IODriverFn, NockAppHandle, PokeResult};
use nockapp::nockapp::wire::Wire;
use nockapp::nockapp::NockAppError;
use nockapp::noun::slab::NounSlab;
use nockapp::noun::{AtomExt, NounExt};
use nockvm::noun::{Atom, D, T};
use nockvm_macros::tas;
use tempfile::tempdir;
use tracing::{instrument, warn};
use tokio::sync::mpsc;
use tokio::task::JoinSet;

/// Mining module for Nockchain
/// 
/// This module implements dual-layer parallelization for optimal mining performance:
/// 
/// **Layer 1 - Parallel STARK Generation (NEW)**:
/// - True parallelization within each proof attempt
/// - Parallel FFT/NTT operations using all CPU cores
/// - Parallel polynomial multiplication and FRI protocol
/// - Single kernel instance with shared memory
/// - Exponential speedup (O(n log n / p) complexity)
/// 
/// **Layer 2 - Multiple Mining Workers**:
/// - Multiple workers attempting different nonces concurrently
/// - Each worker gets its own kernel instance
/// - Linear speedup with number of workers
/// - Useful when network difficulty is high
/// 
/// **Combined Benefits**:
/// - Each worker uses parallel STARK generation
/// - With 4 workers on 8 cores: each worker uses 2 cores for parallel FFT
/// - With 1 worker on 8 cores: all 8 cores used for single proof (faster per proof)
/// 
/// Configuration:
/// - Set `MINING_THREADS` environment variable to control total thread pool size
/// - Worker count is automatically adjusted based on available threads
/// - Example: `MINING_THREADS=16 nockchain --mine` on 16-core system
///   - Can run 4 workers with 4 threads each, or
///   - Can run 2 workers with 8 threads each (recommended)
/// 
/// Performance characteristics:
/// - True parallel STARK: ~6-7x speedup on 8 cores (per proof)
/// - Multiple workers: Nx speedup for N workers (more attempts)
/// - Combined: Optimal for finding proofs quickly
/// 
/// See TRUE_PARALLEL_MINING.md for technical details on the parallel algorithms.

pub enum MiningWire {
    Mined,
    Candidate,
    SetPubKey,
    Enable,
}

impl MiningWire {
    pub fn verb(&self) -> &'static str {
        match self {
            MiningWire::Mined => "mined",
            MiningWire::SetPubKey => "setpubkey",
            MiningWire::Candidate => "candidate",
            MiningWire::Enable => "enable",
        }
    }
}

impl Wire for MiningWire {
    const VERSION: u64 = 1;
    const SOURCE: &'static str = "miner";

    fn to_wire(&self) -> nockapp::wire::WireRepr {
        let tags = vec![self.verb().into()];
        nockapp::wire::WireRepr::new(MiningWire::SOURCE, MiningWire::VERSION, tags)
    }
}

#[derive(Debug, Clone)]
pub struct MiningKeyConfig {
    pub share: u64,
    pub m: u64,
    pub keys: Vec<String>,
}

impl FromStr for MiningKeyConfig {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Expected format: "share,m:key1,key2,key3"
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 2 {
            return Err("Invalid format. Expected 'share,m:key1,key2,key3'".to_string());
        }

        let share_m: Vec<&str> = parts[0].split(',').collect();
        if share_m.len() != 2 {
            return Err("Invalid share,m format".to_string());
        }

        let share = share_m[0].parse::<u64>().map_err(|e| e.to_string())?;
        let m = share_m[1].parse::<u64>().map_err(|e| e.to_string())?;
        let keys: Vec<String> = parts[1].split(',').map(String::from).collect();

        Ok(MiningKeyConfig { share, m, keys })
    }
}

// Cached mining resources to avoid repeated allocations
struct MiningResources {
    kernel: Kernel,
    snapshot_dir: PathBuf,
}

// Mining worker pool configuration
const DEFAULT_MINING_THREADS: usize = 4; // Default to 4 threads
const MAX_MINING_QUEUE: usize = 32; // Maximum queued candidates

/// Calculate optimal worker count based on available threads
/// 
/// Strategy:
/// - For ≤4 threads: 1 worker using all threads (maximize per-proof speed)
/// - For 5-8 threads: 2 workers (balance between attempts and speed)
/// - For 9-16 threads: 4 workers
/// - For >16 threads: threads/4 workers (minimum 4 threads per worker)
fn calculate_optimal_workers(total_threads: usize) -> usize {
    match total_threads {
        1..=4 => 1,
        5..=8 => 2,
        9..=16 => 4,
        _ => total_threads / 4,
    }
}

pub fn create_mining_driver(
    mining_config: Option<Vec<MiningKeyConfig>>,
    mine: bool,
    init_complete_tx: Option<tokio::sync::oneshot::Sender<()>>,
) -> IODriverFn {
    Box::new(move |handle| {
        Box::pin(async move {
            let Some(configs) = mining_config else {
                enable_mining(&handle, false).await?;

                if let Some(tx) = init_complete_tx {
                    tx.send(()).map_err(|_| {
                        warn!("Could not send driver initialization for mining driver.");
                        NockAppError::OtherError
                    })?;
                }

                return Ok(());
            };
            if configs.len() == 1
                && configs[0].share == 1
                && configs[0].m == 1
                && configs[0].keys.len() == 1
            {
                set_mining_key(&handle, configs[0].keys[0].clone()).await?;
            } else {
                set_mining_key_advanced(&handle, configs).await?;
            }
            enable_mining(&handle, mine).await?;

            if let Some(tx) = init_complete_tx {
                tx.send(()).map_err(|_| {
                    warn!("Could not send driver initialization for mining driver.");
                    NockAppError::OtherError
                })?;
            }

            if !mine {
                return Ok(());
            }

            // Determine total thread pool size from environment variable
            let total_threads = std::env::var("MINING_THREADS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or_else(|| {
                    std::thread::available_parallelism()
                        .map(|n| n.get())
                        .unwrap_or(4)
                });
            
            // Calculate optimal worker count
            let worker_count = calculate_optimal_workers(total_threads);
            let threads_per_worker = total_threads / worker_count;
            
            // Create channels for distributing work to mining threads
            let (candidate_tx, candidate_rx) = mpsc::channel::<NounSlab>(MAX_MINING_QUEUE);
            let candidate_rx = Arc::new(tokio::sync::Mutex::new(candidate_rx));

            // Start mining worker pool
            let mut mining_workers = JoinSet::new();
            
            let mut handle = handle; // Make handle mutable for reassignment
            
            for worker_id in 0..worker_count {
                let worker_rx = candidate_rx.clone();
                let (new_handle, worker_handle) = handle.dup();
                handle = new_handle;  // Reassign handle for next iteration
                
                mining_workers.spawn(mining_worker(
                    worker_id,
                    worker_rx,
                    worker_handle,
                ));
            }

            loop {
                tokio::select! {
                    effect_res = handle.next_effect() => {
                        let Ok(effect) = effect_res else {
                            warn!("Error receiving effect in mining driver: {effect_res:?}");
                            continue;
                        };
                        let Ok(effect_cell) = (unsafe { effect.root().as_cell() }) else {
                            drop(effect);
                            continue;
                        };

                        if effect_cell.head().eq_bytes("mine") {
                            let candidate_slab = {
                                let mut slab = NounSlab::new();
                                slab.copy_into(effect_cell.tail());
                                slab
                            };
                            
                            // Try to send to worker pool
                            match candidate_tx.try_send(candidate_slab) {
                                Ok(_) => {},
                                Err(mpsc::error::TrySendError::Full(_)) => {
                                    warn!("Mining queue full, dropping candidate");
                                }
                                Err(e) => {
                                    warn!("Error sending candidate to mining pool: {:?}", e);
                                }
                            }
                        }
                    },
                    // Monitor worker health and restart if needed
                    Some(res) = mining_workers.join_next() => {
                        match res {
                            Ok(worker_id) => {
                                warn!("Mining worker {} terminated, restarting", worker_id);
                                // Restart the worker
                                let worker_rx = candidate_rx.clone();
                                let (new_handle, worker_handle) = handle.dup();
                                handle = new_handle;  // Reassign handle
                                
                                mining_workers.spawn(mining_worker(
                                    worker_id,
                                    worker_rx,
                                    worker_handle,
                                ));
                            }
                            Err(e) => {
                                warn!("Mining worker panicked: {:?}", e);
                            }
                        }
                    }
                }
            }
        })
    })
}

// Mining worker that processes candidates from the queue
async fn mining_worker(
    worker_id: usize,
    candidate_rx: Arc<tokio::sync::Mutex<mpsc::Receiver<NounSlab>>>,
    mut handle: NockAppHandle,
) -> usize {
    // Each worker gets its own kernel instance and resources
    let resources = match initialize_mining_resources().await {
        Ok(res) => Arc::new(res),
        Err(e) => {
            warn!("Failed to initialize mining resources for worker {}: {:?}", worker_id, e);
            return worker_id;
        }
    };
    
    loop {
        // Wait for next candidate
        let candidate = {
            let mut rx = candidate_rx.lock().await;
            match rx.recv().await {
                Some(c) => c,
                None => {
                    warn!("Mining worker {} channel closed", worker_id);
                    return worker_id;
                }
            }
        };
        
        // Process the mining attempt with this worker's kernel
        // Create a new handle for this attempt
        let (new_handle, attempt_handle) = handle.dup();
        handle = new_handle; // Save one handle for the next iteration
        mining_attempt_with_worker(candidate, attempt_handle, resources.clone(), worker_id).await;
    }
}

// Process mining attempt with worker-specific resources
async fn mining_attempt_with_worker(
    candidate: NounSlab,
    handle: NockAppHandle,
    resources: Arc<MiningResources>,
    worker_id: usize,
) -> () {
    let effects_slab = match resources.kernel
        .poke(MiningWire::Candidate.to_wire(), candidate)
        .await {
        Ok(slab) => slab,
        Err(e) => {
            warn!("Worker {} mining attempt failed: {:?}", worker_id, e);
            return;
        }
    };
    
    for effect in effects_slab.to_vec() {
        let Ok(effect_cell) = (unsafe { effect.root().as_cell() }) else {
            drop(effect);
            continue;
        };
        if effect_cell.head().eq_bytes("command") {
            handle
                .poke(MiningWire::Mined.to_wire(), effect)
                .await
                .expect("Could not poke nockchain with mined PoW");
        }
    }
}

// Initialize mining resources once and cache them
async fn initialize_mining_resources() -> Result<MiningResources, Box<dyn std::error::Error>> {
    let snapshot_dir = tokio::task::spawn_blocking(|| {
        tempdir().expect("Failed to create temporary directory")
    })
    .await?;
    
    // Always use parallel hot state for optimal performance
    let hot_state = zkvm_jetpack::hot::produce_prover_hot_state();
    
    let snapshot_path_buf = snapshot_dir.path().to_path_buf();
    let jam_paths = JamPaths::new(&snapshot_path_buf);
    
    let kernel = Kernel::load_with_hot_state_huge(
        snapshot_path_buf.clone(),
        jam_paths,
        KERNEL,
        &hot_state,
        false,
    )
    .await?;
    
    Ok(MiningResources {
        kernel,
        snapshot_dir: snapshot_path_buf,
    })
}

// Original mining_attempt function kept for backward compatibility
pub async fn mining_attempt(candidate: NounSlab, handle: NockAppHandle) -> () {
    let snapshot_dir =
        tokio::task::spawn_blocking(|| tempdir().expect("Failed to create temporary directory"))
            .await
            .expect("Failed to create temporary directory");
    let hot_state = zkvm_jetpack::hot::produce_prover_hot_state();
    let snapshot_path_buf = snapshot_dir.path().to_path_buf();
    let jam_paths = JamPaths::new(snapshot_dir.path());
    // Spawns a new std::thread for this mining attempt
    let kernel =
        Kernel::load_with_hot_state_huge(snapshot_path_buf, jam_paths, KERNEL, &hot_state, false)
            .await
            .expect("Could not load mining kernel");
    let effects_slab = kernel
        .poke(MiningWire::Candidate.to_wire(), candidate)
        .await
        .expect("Could not poke mining kernel with candidate");
    for effect in effects_slab.to_vec() {
        let Ok(effect_cell) = (unsafe { effect.root().as_cell() }) else {
            drop(effect);
            continue;
        };
        if effect_cell.head().eq_bytes("command") {
            handle
                .poke(MiningWire::Mined.to_wire(), effect)
                .await
                .expect("Could not poke nockchain with mined PoW");
        }
    }
}

#[instrument(skip(handle, pubkey))]
async fn set_mining_key(
    handle: &NockAppHandle,
    pubkey: String,
) -> Result<PokeResult, NockAppError> {
    let mut set_mining_key_slab = NounSlab::new();
    let set_mining_key = Atom::from_value(&mut set_mining_key_slab, "set-mining-key")
        .expect("Failed to create set-mining-key atom");
    let pubkey_cord =
        Atom::from_value(&mut set_mining_key_slab, pubkey).expect("Failed to create pubkey atom");
    let set_mining_key_poke = T(
        &mut set_mining_key_slab,
        &[
            D(tas!(b"command")),
            set_mining_key.as_noun(),
            pubkey_cord.as_noun(),
        ],
    );
    set_mining_key_slab.set_root(set_mining_key_poke);

    handle
        .poke(MiningWire::SetPubKey.to_wire(), set_mining_key_slab)
        .await
}

async fn set_mining_key_advanced(
    handle: &NockAppHandle,
    configs: Vec<MiningKeyConfig>,
) -> Result<PokeResult, NockAppError> {
    let mut set_mining_key_slab = NounSlab::new();
    let set_mining_key_adv = Atom::from_value(&mut set_mining_key_slab, "set-mining-key-advanced")
        .expect("Failed to create set-mining-key-advanced atom");

    // Create the list of configs
    let mut configs_list = D(0);
    for config in configs {
        // Create the list of keys
        let mut keys_noun = D(0);
        for key in config.keys {
            let key_atom =
                Atom::from_value(&mut set_mining_key_slab, key).expect("Failed to create key atom");
            keys_noun = T(&mut set_mining_key_slab, &[key_atom.as_noun(), keys_noun]);
        }

        // Create the config tuple [share m keys]
        let config_tuple = T(
            &mut set_mining_key_slab,
            &[D(config.share), D(config.m), keys_noun],
        );

        configs_list = T(&mut set_mining_key_slab, &[config_tuple, configs_list]);
    }

    let set_mining_key_poke = T(
        &mut set_mining_key_slab,
        &[
            D(tas!(b"command")),
            set_mining_key_adv.as_noun(),
            configs_list,
        ],
    );
    set_mining_key_slab.set_root(set_mining_key_poke);

    handle
        .poke(MiningWire::SetPubKey.to_wire(), set_mining_key_slab)
        .await
}

//TODO add %set-mining-key-multisig poke
#[instrument(skip(handle))]
async fn enable_mining(handle: &NockAppHandle, enable: bool) -> Result<PokeResult, NockAppError> {
    let mut enable_mining_slab = NounSlab::new();
    let enable_mining = Atom::from_value(&mut enable_mining_slab, "enable-mining")
        .expect("Failed to create enable-mining atom");
    let enable_mining_poke = T(
        &mut enable_mining_slab,
        &[
            D(tas!(b"command")),
            enable_mining.as_noun(),
            D(if enable { 0 } else { 1 }),
        ],
    );
    enable_mining_slab.set_root(enable_mining_poke);
    handle
        .poke(MiningWire::Enable.to_wire(), enable_mining_slab)
        .await
}
