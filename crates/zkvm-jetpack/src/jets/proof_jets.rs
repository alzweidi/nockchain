use nockvm::interpreter::Context;
use nockvm::jets::util::slot;
use nockvm::jets::Result;
use nockvm::noun::{Atom, Cell, IndirectAtom, Noun, D, T};
use rayon::prelude::*;

use crate::jets::utils::jet_err;
use crate::noun::noun_ext::NounExt;

/// Parallel proof generation jet this will probs not work but fuck it we gon try anyway
/// 
/// Takes: [length block-commitment nonces-list]
/// Returns: (unit [proof dig nonce]) - first valid proof or ~
pub fn prove_block_parallel_jet(context: &mut Context, subject: Noun) -> Result {
    let sam = slot(subject, 6)?;
    
    // Extract arguments
    let length_noun = slot(sam, 2)?;
    let block_commitment_noun = slot(sam, 6)?;
    let nonces_list_noun = slot(sam, 7)?;
    
    // Parse length
    let Ok(length_atom) = length_noun.as_atom() else {
        return jet_err();
    };
    let Ok(length) = length_atom.as_u64() else {
        return jet_err();
    };
    
    // Parse block commitment (32 bytes)
    let Ok(commitment_atom) = block_commitment_noun.as_atom() else {
        return jet_err();
    };
    let commitment_bytes = commitment_atom.as_bytes();
    if commitment_bytes.len() != 32 {
        return jet_err();
    };
    let mut block_commitment = [0u8; 32];
    block_commitment.copy_from_slice(&commitment_bytes);
    
    // Parse nonces list
    let mut nonces = Vec::new();
    let mut current = nonces_list_noun;
    loop {
        if unsafe { current.raw_equals(D(0)) } {
            break;
        }
        
        let Ok(cell) = unsafe { current.as_cell() } else {
            return jet_err();
        };
        
        let Ok(nonce_atom) = cell.head().as_atom() else {
            return jet_err();
        };
        
        let nonce_bytes = nonce_atom.as_bytes();
        if nonce_bytes.len() != 32 {
            return jet_err();
        };
        
        let mut nonce = [0u8; 32];
        nonce.copy_from_slice(&nonce_bytes);
        nonces.push(nonce);
        
        current = cell.tail();
    }
    
    if nonces.is_empty() {
        return Ok(D(0)); // Return ~ for empty list
    }
    
    // Process nonces in parallel
    let results: Vec<_> = nonces
        .par_iter()
        .enumerate()
        .map(|(idx, nonce)| {
            // Call the existing proof generation logic
            // This is where we interface with the STARK prover
            match generate_proof_for_nonce(length, &block_commitment, nonce) {
                Some((proof, hash)) => Some((idx, *nonce, proof, hash)),
                None => None,
            }
        })
        .collect();
    
    // Find first valid proof
    for result in results {
        if let Some((idx, nonce, proof, hash)) = result {
            // Check if proof meets difficulty target
            // TODO: Get target from somewhere
            
            // Build result: [0 [proof hash nonce]]
            let mut result_slab = context.stack.alloc_slab();
            
            // Convert proof to noun
            let proof_noun = proof_to_noun(&mut result_slab, &proof);
            
            // Convert hash to atom
            let hash_atom = Atom::from_bytes(&mut result_slab, &hash)
                .expect("Failed to create hash atom");
            
            // Convert nonce to atom
            let nonce_atom = Atom::from_bytes(&mut result_slab, &nonce)
                .expect("Failed to create nonce atom");
            
            // Build tuple [proof hash nonce]
            let tuple = T(&mut result_slab, &[proof_noun, hash_atom.as_noun(), nonce_atom.as_noun()]);
            
            // Return [0 tuple] (some)
            let result = T(&mut result_slab, &[D(0), tuple]);
            
            return Ok(result);
        }
    }
    
    // No valid proof found, return ~
    Ok(D(0))
}

// Call the existing proof generation logic
// This is a simplified version - the real implementation needs to properly
// interface with the existing STARK prover
fn generate_proof_for_nonce(
    length: u64,
    block_commitment: &[u8; 32],
    nonce: &[u8; 32],
) -> Option<(Vec<u8>, [u8; 32])> {
    // For now, return None - we need to wire this up to the actual prover
    // The real implementation would:
    // 1. Create the puzzle from block_commitment and nonce
    // 2. Run the STARK prover
    // 3. Generate the proof
    // 4. Calculate proof hash
    // 5. Check if it meets difficulty
    
    // This is where we'd call into the existing proof generation
    // but that requires more integration work
    None
}

// Convert proof bytes to noun representation
fn proof_to_noun(slab: &mut nockvm::mem::NockStack, proof: &[u8]) -> Noun {
    // TODO: Implement proper proof serialization
    // For now, just convert to atom
    Atom::from_bytes(slab, proof)
        .expect("Failed to create proof atom")
        .as_noun()
} 
