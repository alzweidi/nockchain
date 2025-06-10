use anyhow::Result;
use nockapp::{
    noun::{Noun, slot},
    stack::NockStack,
};
use nockvm::{jets::jet_err, noun::IndirectAtom, stack::Context};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use lazy_static::lazy_static;

use crate::jets::bp_jets::{BPolySlice, Belt, bp_shift, finalize_poly, new_handle_mut_slice};
use crate::jets::utils::new_handle_mut_poly_slice;

/// Field element type matching the Hoon belt type
type FieldElement = u64;

/// Cached domain data for a specific domain size
#[derive(Clone, Debug)]
pub struct DomainData {
    /// Powers of the offset element [1, c, c^2, ..., c^(n-1)]
    /// For offset=1, this is just [1, 1, 1, ..., 1]
    pub powers: Vec<Belt>,
    
    /// FFT twiddle factors for this domain size
    pub twiddle_factors: Vec<Belt>,
    
    /// Inverse FFT twiddle factors
    pub inv_twiddles: Vec<Belt>,
}

/// Global domain cache shared across all mining attempts
lazy_static! {
    static ref DOMAIN_CACHE: Arc<Mutex<DomainCache>> = Arc::new(Mutex::new(DomainCache::new()));
}

/// Cache for pre-computed domain elements used in polynomial interpolation
pub struct DomainCache {
    /// Map from (domain_size, offset) -> precomputed domain data
    /// For offset=1 (the common case), we can heavily cache
    domains: HashMap<(usize, usize), DomainData>,
    
    /// Cache statistics
    hits: u64,
    misses: u64,
    bp_shift_calls: u64,
    bp_intercosate_calls: u64,
}

impl DomainCache {
    /// Create a new empty domain cache
    pub fn new() -> Self {
        let mut cache = DomainCache {
            domains: HashMap::new(),
            hits: 0,
            misses: 0,
            bp_shift_calls: 0,
            bp_intercosate_calls: 0,
        };
        
        // Pre-populate common domain sizes used in STARK proofs
        eprintln!("[DOMAIN_CACHE] Initializing with common domain sizes...");
        for size_pow in 5..=12 {  // 2^5 = 32 to 2^12 = 4096
            let size = 1 << size_pow;
            cache.precompute_domain(size, 1); // offset is typically 1 in mining
            eprintln!("[DOMAIN_CACHE] Precomputed domain size {} with offset 1", size);
        }
        
        eprintln!("[DOMAIN_CACHE] Initialization complete - {} domains precomputed", cache.domains.len());
        cache
    }
    
    /// Precompute and cache domain data for a given size and offset
    fn precompute_domain(&mut self, size: usize, offset: usize) {
        let key = (size, offset);
        
        // Skip if already cached
        if self.domains.contains_key(&key) {
            eprintln!("[DOMAIN_CACHE] Domain ({}, {}) already cached", size, offset);
            return;
        }
        
        eprintln!("[DOMAIN_CACHE] Computing domain ({}, {})...", size, offset);
        
        // Compute powers of offset
        let mut powers = Vec::with_capacity(size);
        let mut power = Belt::one();
        let offset_belt = Belt(offset as u64);
        
        for i in 0..size {
            powers.push(power);
            power = power * offset_belt;
            if i % 100 == 0 && i > 0 {
                eprintln!("[DOMAIN_CACHE]   Computed {} powers...", i);
            }
        }
        
        // For now, placeholder twiddle factors
        // In full implementation, these would be nth roots of unity
        let twiddle_factors = powers.clone();
        let inv_twiddles = powers.clone();
        
        let data = DomainData {
            powers,
            twiddle_factors,
            inv_twiddles,
        };
        
        self.domains.insert((size, offset), data);
        eprintln!("[DOMAIN_CACHE] Domain ({}, {}) cached successfully", size, offset);
    }
    
    /// Get cached domain data
    pub fn get(&mut self, size: usize, offset: usize) -> Option<&DomainData> {
        let key = (size, offset);
        if self.domains.contains_key(&key) {
            self.hits += 1;
            eprintln!("[DOMAIN_CACHE] Cache HIT for domain ({}, {}) - Total hits: {}", size, offset, self.hits);
            self.domains.get(&key)
        } else {
            self.misses += 1;
            eprintln!("[DOMAIN_CACHE] Cache MISS for domain ({}, {}) - Total misses: {}", size, offset, self.misses);
            None
        }
    }
}

/// Jet for bp-shift that uses cached domain powers
pub fn bp_shift_cached_jet(context: &mut Context, subject: Noun) -> Result {
    let sam = slot(subject, 6)?;
    let bp = slot(sam, 2)?;
    let c = slot(sam, 3)?;

    let (Ok(bp_poly), Ok(c_belt)) = (BPolySlice::try_from(bp), c.as_belt()) else {
        eprintln!("[DOMAIN_CACHE] bp_shift_cached_jet: Invalid input types");
        return jet_err();
    };
    
    let poly_len = bp_poly.len();
    let offset = c_belt.0 as usize;
    
    // Update call counter
    {
        let mut cache = DOMAIN_CACHE.lock().unwrap();
        cache.bp_shift_calls += 1;
        if cache.bp_shift_calls % 100 == 0 {
            eprintln!("[DOMAIN_CACHE] bp_shift called {} times", cache.bp_shift_calls);
        }
    }
    
    eprintln!("[DOMAIN_CACHE] bp_shift_cached_jet called with poly_len={}, offset={}", poly_len, offset);
    
    // Check cache for precomputed powers
    let mut cache = DOMAIN_CACHE.lock().unwrap();
    
    if let Some(domain_data) = cache.get(poly_len, offset) {
        // Cache hit! Use precomputed powers
        eprintln!("[DOMAIN_CACHE] Using cached powers for bp_shift");
        let powers = &domain_data.powers;
        
        // Allocate result
        let (res_atom, res_poly): (IndirectAtom, &mut [Belt]) =
            new_handle_mut_slice(&mut context.stack, Some(poly_len));
        
        // Multiply polynomial coefficients by cached powers
        for i in 0..poly_len {
            res_poly[i] = bp_poly.0[i] * powers[i];
        }
        
        let res_cell = finalize_poly(&mut context.stack, Some(res_poly.len()), res_atom);
        eprintln!("[DOMAIN_CACHE] bp_shift completed using cache");
        Ok(res_cell)
    } else {
        // Cache miss - fall back to original implementation
        drop(cache); // Release lock before computation
        eprintln!("[DOMAIN_CACHE] Falling back to original bp_shift implementation");
        
        let (res_atom, res_poly): (IndirectAtom, &mut [Belt]) =
            new_handle_mut_slice(&mut context.stack, Some(bp_poly.len()));
        bp_shift(bp_poly.0, &c_belt, res_poly);
        
        let res_cell = finalize_poly(&mut context.stack, Some(res_poly.len()), res_atom);
        eprintln!("[DOMAIN_CACHE] bp_shift completed using fallback");
        Ok(res_cell)
    }
}

/// Jet for bp-intercosate that leverages domain cache
pub fn bp_intercosate_cached_jet(context: &mut Context, subject: Noun) -> Result {
    let sam = slot(subject, 6)?;
    let offset = slot(sam, 2)?;
    let order = slot(sam, 3)?;
    let values = slot(sam, 4)?;
    
    // Extract parameters
    let Ok(offset_belt) = offset.as_belt() else {
        eprintln!("[DOMAIN_CACHE] bp_intercosate_cached_jet: Invalid offset type");
        return jet_err();
    };
    
    let order_u64 = order.as_u64()?;
    
    let Ok(values_poly) = BPolySlice::try_from(values) else {
        eprintln!("[DOMAIN_CACHE] bp_intercosate_cached_jet: Invalid values type");
        return jet_err();
    };
    
    // Validate inputs
    if order_u64 == 0 || (order_u64 & (order_u64 - 1)) != 0 {
        eprintln!("[DOMAIN_CACHE] bp_intercosate_cached_jet: Order {} is not power of 2", order_u64);
        return jet_err(); // Order must be power of 2
    }
    
    if values_poly.len() != order_u64 as usize {
        eprintln!("[DOMAIN_CACHE] bp_intercosate_cached_jet: Values length {} != order {}", values_poly.len(), order_u64);
        return jet_err(); // Values must match order
    }
    
    // Update call counter
    {
        let mut cache = DOMAIN_CACHE.lock().unwrap();
        cache.bp_intercosate_calls += 1;
        if cache.bp_intercosate_calls % 100 == 0 {
            eprintln!("[DOMAIN_CACHE] bp_intercosate called {} times", cache.bp_intercosate_calls);
        }
    }
    
    eprintln!("[DOMAIN_CACHE] bp_intercosate_cached_jet called with order={}, offset={}", order_u64, offset.as_u64()?);
    
    // Check if we can use cached twiddle factors for IFFT
    let cache = DOMAIN_CACHE.lock().unwrap();
    let cache_key = (order_u64 as usize, offset.as_u64()? as usize);
    
    if let Some(domain_data) = cache.domains.get(&cache_key) {
        eprintln!("[DOMAIN_CACHE] Cache HIT for bp_intercosate domain");
        drop(cache); // Release lock early
        
        // Perform IFFT using cached inverse twiddle factors
        let ifft_result = if !domain_data.inv_twiddles.is_empty() {
            eprintln!("[DOMAIN_CACHE] Using cached twiddle factors for IFFT");
            // Use cached twiddle factors
            bp_ntt(values_poly.0, &domain_data.inv_twiddles[0])
        } else {
            eprintln!("[DOMAIN_CACHE] Computing inverse root for IFFT");
            // Fallback: compute inverse root
            let order_belt = Belt(order_u64);
            let inv_root = match order_belt.ordered_root() {
                Ok(root) => root.inv(),
                Err(_) => {
                    eprintln!("[DOMAIN_CACHE] Failed to compute ordered root");
                    return jet_err();
                },
            };
            bp_ntt(values_poly.0, &inv_root)
        };
        
        // Scale by 1/n
        let n_inv = Belt(order_u64).inv();
        let mut scaled_ifft: Vec<Belt> = ifft_result.iter()
            .map(|&coeff| coeff * n_inv)
            .collect();
        
        // Apply shift by c^{-1}
        let offset_inv = offset_belt.inv();
        let mut power = Belt::one();
        for coeff in scaled_ifft.iter_mut() {
            *coeff = *coeff * power;
            power = power * offset_inv;
        }
        
        // Allocate result
        let (res_atom, res_poly): (IndirectAtom, &mut [Belt]) =
            new_handle_mut_slice(&mut context.stack, Some(scaled_ifft.len()));
        
        // Copy result
        res_poly.copy_from_slice(&scaled_ifft);
        
        let res_cell = finalize_poly(&mut context.stack, Some(res_poly.len()), res_atom);
        eprintln!("[DOMAIN_CACHE] bp_intercosate completed using cache");
        Ok(res_cell)
    } else {
        eprintln!("[DOMAIN_CACHE] Cache MISS for bp_intercosate domain");
        drop(cache);
        
        // No cached data - compute from scratch
        let order_belt = Belt(order_u64);
        let inv_root = match order_belt.ordered_root() {
            Ok(root) => root.inv(),
            Err(_) => {
                eprintln!("[DOMAIN_CACHE] Failed to compute ordered root");
                return jet_err();
            },
        };
        
        // Perform IFFT
        eprintln!("[DOMAIN_CACHE] Computing IFFT from scratch");
        let ifft_result = bp_ntt(values_poly.0, &inv_root);
        
        // Scale by 1/n
        let n_inv = order_belt.inv();
        let mut scaled_ifft: Vec<Belt> = ifft_result.iter()
            .map(|&coeff| coeff * n_inv)
            .collect();
        
        // Apply shift by c^{-1}
        let offset_inv = offset_belt.inv();
        let mut power = Belt::one();
        for coeff in scaled_ifft.iter_mut() {
            *coeff = *coeff * power;
            power = power * offset_inv;
        }
        
        // Allocate result
        let (res_atom, res_poly): (IndirectAtom, &mut [Belt]) =
            new_handle_mut_slice(&mut context.stack, Some(scaled_ifft.len()));
        
        // Copy result
        res_poly.copy_from_slice(&scaled_ifft);
        
        let res_cell = finalize_poly(&mut context.stack, Some(res_poly.len()), res_atom);
        eprintln!("[DOMAIN_CACHE] bp_intercosate completed using fallback");
        Ok(res_cell)
    }
}

/// Initialize the domain cache for mining
pub fn init_domain_cache() {
    // Force initialization of the lazy static
    let cache = DOMAIN_CACHE.lock().unwrap();
    eprintln!("[DOMAIN_CACHE] Cache ready with {} entries precomputed", cache.domains.len());
    eprintln!("[DOMAIN_CACHE] Cache will log hits/misses and call statistics during mining");
}

/// Get global cache statistics
pub fn get_cache_stats() -> (u64, u64, u64, u64) {
    let cache = DOMAIN_CACHE.lock().unwrap();
    (cache.hits, cache.misses, cache.bp_shift_calls, cache.bp_intercosate_calls)
}

/// Print detailed cache statistics
pub fn print_cache_stats() {
    let cache = DOMAIN_CACHE.lock().unwrap();
    eprintln!("[DOMAIN_CACHE] ===== Cache Statistics =====");
    eprintln!("[DOMAIN_CACHE] Total domains cached: {}", cache.domains.len());
    eprintln!("[DOMAIN_CACHE] Cache hits: {}", cache.hits);
    eprintln!("[DOMAIN_CACHE] Cache misses: {}", cache.misses);
    eprintln!("[DOMAIN_CACHE] Hit rate: {:.2}%", 
        if cache.hits + cache.misses > 0 {
            (cache.hits as f64 / (cache.hits + cache.misses) as f64) * 100.0
        } else {
            0.0
        }
    );
    eprintln!("[DOMAIN_CACHE] bp_shift calls: {}", cache.bp_shift_calls);
    eprintln!("[DOMAIN_CACHE] bp_intercosate calls: {}", cache.bp_intercosate_calls);
    eprintln!("[DOMAIN_CACHE] ===========================");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_domain_cache_creation() {
        let mut cache = DomainCache::new();
        
        // Check that common sizes are pre-populated with offset=1
        let offset_one = 1;
        assert!(cache.domains.contains_key(&(32, offset_one)));
        assert!(cache.domains.contains_key(&(64, offset_one)));
        assert!(cache.domains.contains_key(&(128, offset_one)));
        assert!(cache.domains.contains_key(&(256, offset_one)));
    }
    
    #[test]
    fn test_cache_hit_miss() {
        let mut cache = DomainCache::new();
        let offset_one = 1;
        
        // First access should be a hit (pre-computed)
        let _ = cache.get(64, offset_one);
        assert_eq!(cache.hits, 1);
        assert_eq!(cache.misses, 0);
        
        // Access uncommon size should be a miss
        let offset_two = 2;
        let _ = cache.get(37, offset_two);
        assert_eq!(cache.hits, 1);
        assert_eq!(cache.misses, 1);
        
        // Second access to same size should be a hit
        let _ = cache.get(37, offset_two);
        assert_eq!(cache.hits, 2);
        assert_eq!(cache.misses, 1);
    }
}
