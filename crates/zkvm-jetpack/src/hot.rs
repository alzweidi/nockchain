use either::Either::*;
use nockvm::jets::hot::{HotEntry, K_138};

use crate::jets::*;
use crate::jets::base_jets::*;
use crate::jets::bp_jets::*;
use crate::jets::bp_jets_parallel;
use crate::jets::cheetah_jets::*;
use crate::jets::crypto_jets::*;
use crate::jets::fext_jets::*;
use crate::jets::mary_jets::*;
use crate::jets::table_jets::build_table_dats_jet;
use crate::jets::tip5_jets::*;
use crate::jets::verifier_jets::*;
use crate::jets::mega_jets::*;

/// Produces hot state with parallel polynomial operations for optimized mining
/// 
/// This uses parallel implementations of FFT/NTT operations which are 
/// the main bottleneck in STARK proof generation.
/// 
/// Benefits:
/// - 4-8x speedup on multi-core systems  
/// - Better CPU utilization during proof generation
/// - Configurable thread count via MINING_THREADS environment variable
pub fn produce_prover_hot_state() -> Vec<HotEntry> {
    // Initialize the thread pool on first use
    static INIT: std::sync::Once = std::sync::Once::new();
    INIT.call_once(|| {
        bp_jets_parallel::init_mining_thread_pool();
    });
    
    let mut jets: Vec<HotEntry> = Vec::new();
    jets.extend(BASE_FIELD_JETS);
    
    // Use parallel polynomial jets for optimal performance
    jets.extend(bp_jets_parallel::registration::PARALLEL_POLY_JETS);
    
    jets.extend(CURVE_JETS);
    jets.extend(ZTD_JETS);
    jets.extend(TABLE_JETS);
    jets.extend(KEYGEN_JETS);
    jets.extend(XTRA_JETS);
    jets.extend(EXTENSION_FIELD_JETS);

    jets
}

/// Legacy function - now just calls the parallel version
pub fn produce_prover_hot_state_parallel() -> Vec<HotEntry> {
    produce_prover_hot_state()
}

pub const TABLE_JETS: &[HotEntry] = &[
    (
        &[
            K_138,
            Left(b"one"),
            Left(b"two"),
            Left(b"tri"),
            Left(b"qua"),
            Left(b"pen"),
            Left(b"zeke"),
            Left(b"ext-field"),
            Left(b"misc-lib"),
            Left(b"proof-lib"),
            Left(b"utils"),
            Left(b"fri"),
            Left(b"table-lib"),
            Left(b"stark-core"),
            Left(b"fock-core"),
            Left(b"pow"),
            Left(b"stark-engine"),
            Left(b"stark-prover"),
            Left(b"build-table-dats"),
        ],
        1,
        build_table_dats_jet,
    ),
];

pub const XTRA_JETS: &[HotEntry] = &[
    (
        &[
            K_138,
            Left(b"one"),
            Left(b"two"),
            Left(b"tri"),
            Left(b"qua"),
            Left(b"pen"),
            Left(b"zeke"),
            Left(b"ave"),
            Left(b"weld"),
        ],
        1,
        mary_weld_jet,
    ),
    (
        &[
            K_138,
            Left(b"one"),
            Left(b"two"),
            Left(b"tri"),
            Left(b"qua"),
            Left(b"pen"),
            Left(b"zeke"),
            Left(b"ave"),
            Left(b"swag"),
        ],
        1,
        mary_swag_jet,
    ),
    (
        &[
            K_138,
            Left(b"one"),
            Left(b"two"),
            Left(b"tri"),
            Left(b"qua"),
            Left(b"pen"),
            Left(b"zeke"),
            Left(b"ext-field"),
            Left(b"misc-lib"),
            Left(b"proof-lib"),
            Left(b"utils"),
            Left(b"fri"),
            Left(b"table-lib"),
            Left(b"stark-core"),
            Left(b"fock-core"),
            Left(b"pow"),
            Left(b"stark-engine"),
            Left(b"stark-verifier"),
            Left(b"evaluate-deep"),
        ],
        1,
        evaluate_deep_jet,
    ),
    (
        &[
            K_138,
            Left(b"one"),
            Left(b"two"),
            Left(b"tri"),
            Left(b"qua"),
            Left(b"pen"),
            Left(b"zeke"),
            Left(b"ave"),
            Left(b"transpose"),
        ],
        1,
        mary_transpose_jet,
    ),
    (
        &[
            K_138,
            Left(b"one"),
            Left(b"two"),
            Left(b"tri"),
            Left(b"qua"),
            Left(b"pen"),
            Left(b"zeke"),
            Left(b"ext-field"),
            Left(b"mp-to-mega"),
            Left(b"mpeval"),
        ],
        1,
        mpeval_jet,
    ),
];

pub const EXTENSION_FIELD_JETS: &[HotEntry] = &[
    (
        &[
            K_138,
            Left(b"one"),
            Left(b"two"),
            Left(b"tri"),
            Left(b"qua"),
            Left(b"pen"),
            Left(b"zeke"),
            Left(b"ext-field"),
            Left(b"bp-shift"),
        ],
        1,
        bp_shift_jet,
    ),
    (
        &[
            K_138,
            Left(b"one"),
            Left(b"two"),
            Left(b"tri"),
            Left(b"qua"),
            Left(b"pen"),
            Left(b"zeke"),
            Left(b"ext-field"),
            Left(b"bp-coseword"),
        ],
        1,
        bp_coseword_jet,
    ),
    (
        &[
            K_138,
            Left(b"one"),
            Left(b"two"),
            Left(b"tri"),
            Left(b"qua"),
            Left(b"pen"),
            Left(b"zeke"),
            Left(b"ext-field"),
            Left(b"fadd"),
        ],
        1,
        fadd_jet,
    ),
    (
        &[
            K_138,
            Left(b"one"),
            Left(b"two"),
            Left(b"tri"),
            Left(b"qua"),
            Left(b"pen"),
            Left(b"zeke"),
            Left(b"ext-field"),
            Left(b"fsub"),
        ],
        1,
        fsub_jet,
    ),
    (
        &[
            K_138,
            Left(b"one"),
            Left(b"two"),
            Left(b"tri"),
            Left(b"qua"),
            Left(b"pen"),
            Left(b"zeke"),
            Left(b"ext-field"),
            Left(b"fneg"),
        ],
        1,
        fneg_jet,
    ),
    (
        &[
            K_138,
            Left(b"one"),
            Left(b"two"),
            Left(b"tri"),
            Left(b"qua"),
            Left(b"pen"),
            Left(b"zeke"),
            Left(b"ext-field"),
            Left(b"fmul"),
        ],
        1,
        fmul_jet,
    ),
    (
        &[
            K_138,
            Left(b"one"),
            Left(b"two"),
            Left(b"tri"),
            Left(b"qua"),
            Left(b"pen"),
            Left(b"zeke"),
            Left(b"ext-field"),
            Left(b"finv"),
        ],
        1,
        finv_jet,
    ),
    (
        &[
            K_138,
            Left(b"one"),
            Left(b"two"),
            Left(b"tri"),
            Left(b"qua"),
            Left(b"pen"),
            Left(b"zeke"),
            Left(b"ext-field"),
            Left(b"fdiv"),
        ],
        1,
        fdiv_jet,
    ),
    (
        &[
            K_138,
            Left(b"one"),
            Left(b"two"),
            Left(b"tri"),
            Left(b"qua"),
            Left(b"pen"),
            Left(b"zeke"),
            Left(b"ext-field"),
            Left(b"fpow"),
        ],
        1,
        fpow_jet,
    ),
    (
        &[
            K_138,
            Left(b"one"),
            Left(b"two"),
            Left(b"tri"),
            Left(b"qua"),
            Left(b"pen"),
            Left(b"zeke"),
            Left(b"ext-field"),
            Left(b"mp-substitute-mega"),
        ],
        1,
        mp_substitute_mega_jet,
    ),
];

pub const BASE_FIELD_JETS: &[HotEntry] = &[
    (
        &[
            K_138,
            Left(b"one"),
            Left(b"two"),
            Left(b"tri"),
            Left(b"qua"),
            Left(b"pen"),
            Left(b"zeke"),
            Left(b"badd"),
        ],
        1,
        badd_jet,
    ),
    (
        &[
            K_138,
            Left(b"one"),
            Left(b"two"),
            Left(b"tri"),
            Left(b"qua"),
            Left(b"pen"),
            Left(b"zeke"),
            Left(b"bsub"),
        ],
        1,
        bsub_jet,
    ),
    (
        &[
            K_138,
            Left(b"one"),
            Left(b"two"),
            Left(b"tri"),
            Left(b"qua"),
            Left(b"pen"),
            Left(b"zeke"),
            Left(b"bneg"),
        ],
        1,
        bneg_jet,
    ),
    (
        &[
            K_138,
            Left(b"one"),
            Left(b"two"),
            Left(b"tri"),
            Left(b"qua"),
            Left(b"pen"),
            Left(b"zeke"),
            Left(b"bmul"),
        ],
        1,
        bmul_jet,
    ),
    (
        &[
            K_138,
            Left(b"one"),
            Left(b"two"),
            Left(b"tri"),
            Left(b"qua"),
            Left(b"pen"),
            Left(b"zeke"),
            Left(b"ordered-root"),
        ],
        1,
        ordered_root_jet,
    ),
    (
        &[
            K_138,
            Left(b"one"),
            Left(b"two"),
            Left(b"tri"),
            Left(b"qua"),
            Left(b"pen"),
            Left(b"zeke"),
            Left(b"bpow"),
        ],
        1,
        bpow_jet,
    ),
];

pub const ZTD_JETS: &[HotEntry] = &[(
    &[
        K_138,
        Left(b"one"),
        Left(b"two"),
        Left(b"tri"),
        Left(b"qua"),
        Left(b"pen"),
        Left(b"zeke"),
        Left(b"ext-field"),
        Left(b"misc-lib"),
        Left(b"tip5-lib"),
        Left(b"permutation"),
    ],
    1,
    permutation_jet,
)];

pub const KEYGEN_JETS: &[HotEntry] = &[(
    &[
        K_138,
        Left(b"one"),
        Left(b"two"),
        Left(b"tri"),
        Left(b"qua"),
        Left(b"pen"),
        Left(b"zeke"),
        Left(b"ext-field"),
        Left(b"misc-lib"),
        Left(b"proof-lib"),
        Left(b"utils"),
        Left(b"fri"),
        Left(b"table-lib"),
        Left(b"stark-core"),
        Left(b"fock-core"),
        Left(b"pow"),
        Left(b"stark-engine"),
        Left(b"zose"),
        Left(b"argon"),
        Left(b"argon2"),
    ],
    1,
    argon2_jet,
)];

pub const CURVE_JETS: &[HotEntry] = &[(
    &[
        K_138,
        Left(b"one"),
        Left(b"two"),
        Left(b"tri"),
        Left(b"qua"),
        Left(b"pen"),
        Left(b"zeke"),
        Left(b"ext-field"),
        Left(b"misc-lib"),
        Left(b"cheetah"),
        Left(b"curve"),
        Left(b"affine"),
        Left(b"ch-scal"),
    ],
    1,
    ch_scal_jet,
)];
