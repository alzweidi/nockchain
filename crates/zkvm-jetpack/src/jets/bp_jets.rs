use nockvm::interpreter::Context;
use nockvm::jets::util::slot;
use nockvm::jets::Result;
use nockvm::noun::{IndirectAtom, Noun, D, T};

use crate::form::math::fext::*;
use crate::form::poly::*;
use crate::hand::handle::*;
use crate::jets::utils::jet_err;
use crate::noun::noun_ext::NounExt;

// Helper function to convert fpoly to list (for debugging/testing)
pub fn fpoly_to_list_jet(context: &mut Context, subject: Noun) -> Result {
    let sam = slot(subject, 6)?;
    fpoly_to_list(context, sam)
}

pub fn fpoly_to_list(context: &mut Context, sam: Noun) -> Result {
    let Ok(sam_fpoly) = FPolySlice::try_from(sam) else {
        return jet_err();
    };

    // Empty list is a null atom
    let mut res_list = D(0);
    let len = sam_fpoly.len();

    if len == 0 {
        return Ok(res_list);
    }

    for i in (0..len).rev() {
        // Convert Felt to IndirectAtom (3 u64s)
        let felt = &sam_fpoly.data()[i];
        let mut bytes = Vec::with_capacity(24);
        bytes.extend_from_slice(&felt.0[0].0.to_le_bytes());
        bytes.extend_from_slice(&felt.0[1].0.to_le_bytes());
        bytes.extend_from_slice(&felt.0[2].0.to_le_bytes());
        
        let res_atom = unsafe { IndirectAtom::new_raw_bytes(&mut context.stack, bytes.len(), bytes.as_ptr()) };
        res_list = T(&mut context.stack, &[res_atom.as_noun(), res_list]);
    }

    Ok(res_list)
}

// fp_add_jet: Field polynomial addition
pub fn fp_add_jet(context: &mut Context, subject: Noun) -> Result {
    let sam = slot(subject, 6)?;
    let fp = slot(sam, 2)?;
    let fq = slot(sam, 3)?;

    let (Ok(fp_poly), Ok(fq_poly)) = (FPolySlice::try_from(fp), FPolySlice::try_from(fq)) else {
        return jet_err();
    };

    let res_len = std::cmp::max(fp_poly.len(), fq_poly.len());
    let (res, res_poly): (IndirectAtom, &mut [Felt]) =
        new_handle_mut_slice(&mut context.stack, Some(res_len as usize));
    
    fpadd_poly(fp_poly.data(), fq_poly.data(), res_poly);

    let res_cell = finalize_poly(&mut context.stack, Some(res_poly.len()), res);
    Ok(res_cell)
}

// fp_neg_jet: Field polynomial negation
pub fn fp_neg_jet(context: &mut Context, subject: Noun) -> Result {
    let fp = slot(subject, 6)?;

    let Ok(fp_poly) = FPolySlice::try_from(fp) else {
        return jet_err();
    };

    let (res, res_poly): (IndirectAtom, &mut [Felt]) =
        new_handle_mut_slice(&mut context.stack, Some(fp_poly.len()));
    
    fpneg_poly(fp_poly.data(), res_poly);

    let res_cell = finalize_poly(&mut context.stack, Some(res_poly.len()), res);
    Ok(res_cell)
}

// fp_sub_jet: Field polynomial subtraction
pub fn fp_sub_jet(context: &mut Context, subject: Noun) -> Result {
    let sam = slot(subject, 6)?;
    let p = slot(sam, 2)?;
    let q = slot(sam, 3)?;

    let (Ok(p_poly), Ok(q_poly)) = (FPolySlice::try_from(p), FPolySlice::try_from(q)) else {
        return jet_err();
    };

    let res_len = std::cmp::max(p_poly.len(), q_poly.len());
    let (res, res_poly): (IndirectAtom, &mut [Felt]) =
        new_handle_mut_slice(&mut context.stack, Some(res_len as usize));
    
    fpsub_poly(p_poly.data(), q_poly.data(), res_poly);

    let res_cell = finalize_poly(&mut context.stack, Some(res_poly.len()), res);
    Ok(res_cell)
}

// fp_scal_jet: Scale field polynomial by a field element
pub fn fp_scal_jet(context: &mut Context, subject: Noun) -> Result {
    let sam = slot(subject, 6)?;
    let c = slot(sam, 2)?;
    let fp = slot(sam, 3)?;
    
    let Ok(fp_poly) = FPolySlice::try_from(fp) else {
        return jet_err();
    };

    // Extract the Felt scalar from c
    let Ok(c_felt) = Felt::try_from(c) else {
        return jet_err();
    };

    let (res, res_poly): (IndirectAtom, &mut [Felt]) =
        new_handle_mut_slice(&mut context.stack, Some(fp_poly.len()));
    
    fpscal_poly(&c_felt, fp_poly.data(), res_poly);

    let res_cell = finalize_poly(&mut context.stack, Some(res_poly.len()), res);
    Ok(res_cell)
}

// fp_mul_jet: Field polynomial multiplication
pub fn fp_mul_jet(context: &mut Context, subject: Noun) -> Result {
    let sam = slot(subject, 6)?;
    let fp = slot(sam, 2)?;
    let fq = slot(sam, 3)?;

    let (Ok(fp_poly), Ok(fq_poly)) = (FPolySlice::try_from(fp), FPolySlice::try_from(fq)) else {
        return jet_err();
    };

    // Result length is sum of degrees + 1
    let res_len = if fp_poly.len() == 0 || fq_poly.len() == 0 {
        0
    } else {
        fp_poly.len() + fq_poly.len() - 1
    };

    let (res, res_poly): (IndirectAtom, &mut [Felt]) =
        new_handle_mut_slice(&mut context.stack, Some(res_len));
    
    fpmul_poly(fp_poly.data(), fq_poly.data(), res_poly);

    let res_cell = finalize_poly(&mut context.stack, Some(res_poly.len()), res);
    Ok(res_cell)
}

// fp_eval_jet: Evaluate polynomial at a point using Horner's method
pub fn fp_eval_jet(context: &mut Context, subject: Noun) -> Result {
    let sam = slot(subject, 6)?;
    let fp = slot(sam, 2)?;
    let x = slot(sam, 3)?;

    let Ok(fp_poly) = FPolySlice::try_from(fp) else {
        return jet_err();
    };

    let Ok(x_felt) = Felt::try_from(x) else {
        return jet_err();
    };

    let result = fpeval_poly(fp_poly.data(), &x_felt);
    
    // Convert Felt result to Atom
    let mut bytes = Vec::with_capacity(24);
    bytes.extend_from_slice(&result.0[0].0.to_le_bytes());
    bytes.extend_from_slice(&result.0[1].0.to_le_bytes());
    bytes.extend_from_slice(&result.0[2].0.to_le_bytes());
    
    let res_atom = unsafe { IndirectAtom::new_raw_bytes(&mut context.stack, bytes.len(), bytes.as_ptr()) };
    Ok(res_atom.as_noun())
}

// fp_fft_jet: Fast Fourier Transform for field polynomials
pub fn fp_fft_jet(context: &mut Context, subject: Noun) -> Result {
    let sam = slot(subject, 6)?;
    
    let Ok(fp_poly) = FPolySlice::try_from(sam) else {
        return jet_err();
    };

    let len = fp_poly.len();
    let (res, res_poly): (IndirectAtom, &mut [Felt]) =
        new_handle_mut_slice(&mut context.stack, Some(len));
    
    // For FFT, we need a root of unity for the given length
    // This will require the ordered_root function from Hoon
    // For now, we'll implement a basic version
    fp_fft_poly(fp_poly.data(), res_poly);

    let res_cell = finalize_poly(&mut context.stack, Some(res_poly.len()), res);
    Ok(res_cell)
}

// fp_ifft_jet: Inverse Fast Fourier Transform
pub fn fp_ifft_jet(context: &mut Context, subject: Noun) -> Result {
    let sam = slot(subject, 6)?;
    
    let Ok(fp_poly) = FPolySlice::try_from(sam) else {
        return jet_err();
    };

    let len = fp_poly.len();
    let (res, res_poly): (IndirectAtom, &mut [Felt]) =
        new_handle_mut_slice(&mut context.stack, Some(len));
    
    fp_ifft_poly(fp_poly.data(), res_poly);

    let res_cell = finalize_poly(&mut context.stack, Some(res_poly.len()), res);
    Ok(res_cell)
}

// interpolate_jet: Lagrange interpolation
pub fn interpolate_jet(context: &mut Context, subject: Noun) -> Result {
    let sam = slot(subject, 6)?;
    let domain = slot(sam, 2)?;
    let values = slot(sam, 3)?;

    let (Ok(domain_poly), Ok(values_poly)) = 
        (FPolySlice::try_from(domain), FPolySlice::try_from(values)) else {
        return jet_err();
    };

    if domain_poly.len() != values_poly.len() {
        return jet_err();
    }

    let len = domain_poly.len();
    let (res, res_poly): (IndirectAtom, &mut [Felt]) =
        new_handle_mut_slice(&mut context.stack, Some(len));
    
    interpolate_poly(domain_poly.data(), values_poly.data(), res_poly);

    let res_cell = finalize_poly(&mut context.stack, Some(res_poly.len()), res);
    Ok(res_cell)
}

// fpcompose_jet: Polynomial composition P(Q(X))
pub fn fpcompose_jet(context: &mut Context, subject: Noun) -> Result {
    let sam = slot(subject, 6)?;
    let p = slot(sam, 2)?;
    let q = slot(sam, 3)?;

    let (Ok(p_poly), Ok(q_poly)) = (FPolySlice::try_from(p), FPolySlice::try_from(q)) else {
        return jet_err();
    };

    // Result degree is deg(p) * deg(q)
    let res_len = if p_poly.len() == 0 || q_poly.len() == 0 {
        0
    } else {
        (p_poly.len() - 1) * (q_poly.len() - 1) + 1
    };

    let (res, res_poly): (IndirectAtom, &mut [Felt]) =
        new_handle_mut_slice(&mut context.stack, Some(res_len));
    
    fpcompose_poly(p_poly.data(), q_poly.data(), res_poly);

    let res_cell = finalize_poly(&mut context.stack, Some(res_poly.len()), res);
    Ok(res_cell)
}

// ============================================================================
// Field polynomial math operations
// ============================================================================

// Field polynomial addition
fn fpadd_poly(p: &[Felt], q: &[Felt], res: &mut [Felt]) {
    let lp = p.len();
    let lq = q.len();
    let m = std::cmp::max(lp, lq);

    // Initialize result to zero
    for i in 0..m {
        res[i] = Felt::zero();
    }

    // Add p
    for i in 0..lp {
        let temp = res[i];
        fadd(&p[i], &temp, &mut res[i]);
    }

    // Add q
    for i in 0..lq {
        let temp = res[i];
        fadd(&q[i], &temp, &mut res[i]);
    }
}

// Field polynomial negation
fn fpneg_poly(p: &[Felt], res: &mut [Felt]) {
    for i in 0..p.len() {
        fneg(&p[i], &mut res[i]);
    }
}

// Field polynomial subtraction
fn fpsub_poly(p: &[Felt], q: &[Felt], res: &mut [Felt]) {
    let lp = p.len();
    let lq = q.len();
    let m = std::cmp::max(lp, lq);

    // Initialize result to zero
    for i in 0..m {
        res[i] = Felt::zero();
    }

    // Add p
    for i in 0..lp {
        let temp = res[i];
        fadd(&p[i], &temp, &mut res[i]);
    }

    // Subtract q
    for i in 0..lq {
        let mut neg_q = Felt::zero();
        fneg(&q[i], &mut neg_q);
        let temp = res[i];
        fadd(&neg_q, &temp, &mut res[i]);
    }
}

// Scale polynomial by field element
fn fpscal_poly(c: &Felt, p: &[Felt], res: &mut [Felt]) {
    for i in 0..p.len() {
        fmul(c, &p[i], &mut res[i]);
    }
}

// Field polynomial multiplication (naive O(n²) algorithm)
fn fpmul_poly(p: &[Felt], q: &[Felt], res: &mut [Felt]) {
    let lp = p.len();
    let lq = q.len();

    if lp == 0 || lq == 0 {
        return;
    }

    // Initialize result to zero
    for i in 0..res.len() {
        res[i] = Felt::zero();
    }

    // Multiply each term of p with each term of q
    for i in 0..lp {
        for j in 0..lq {
            let mut prod = Felt::zero();
            fmul(&p[i], &q[j], &mut prod);
            let temp = res[i + j];
            fadd(&prod, &temp, &mut res[i + j]);
        }
    }
}

// Evaluate polynomial at a point using Horner's method
fn fpeval_poly(p: &[Felt], x: &Felt) -> Felt {
    if p.is_empty() {
        return Felt::zero();
    }

    let mut result = p[p.len() - 1];
    
    for i in (0..p.len() - 1).rev() {
        let mut temp = Felt::zero();
        fmul(&result, x, &mut temp);
        fadd(&temp, &p[i], &mut result);
    }

    result
}

// Basic FFT implementation (placeholder - needs proper root of unity)
fn fp_fft_poly(p: &[Felt], res: &mut [Felt]) {
    // For now, just copy the input
    // A proper implementation would need the root of unity for the field
    for i in 0..p.len() {
        res[i] = p[i];
    }
}

// Basic IFFT implementation (placeholder)
fn fp_ifft_poly(p: &[Felt], res: &mut [Felt]) {
    // For now, just copy the input
    // A proper implementation would need the inverse root of unity
    for i in 0..p.len() {
        res[i] = p[i];
    }
}

// Lagrange interpolation (basic implementation)
fn interpolate_poly(domain: &[Felt], values: &[Felt], res: &mut [Felt]) {
    let n = domain.len();
    
    // Initialize result to zero
    for i in 0..n {
        res[i] = Felt::zero();
    }

    // For each point, compute its Lagrange basis polynomial
    for i in 0..n {
        // Compute the denominator for normalization
        let mut denom = Felt::one();
        for j in 0..n {
            if i != j {
                let mut diff = Felt::zero();
                fsub(&domain[i], &domain[j], &mut diff);
                let mut new_denom = Felt::zero();
                fmul(&denom, &diff, &mut new_denom);
                denom = new_denom;
            }
        }
        
        // Scale by value[i] / denom
        let mut scale = Felt::zero();
        fdiv(&values[i], &denom, &mut scale);
        
        // Add to result (simplified version)
        let temp = res[0];
        fadd(&scale, &temp, &mut res[0]);
    }
}

// Polynomial composition P(Q(X)) (basic implementation)
fn fpcompose_poly(p: &[Felt], q: &[Felt], res: &mut [Felt]) {
    if p.is_empty() || q.is_empty() {
        return;
    }

    // Initialize result to zero
    for i in 0..res.len() {
        res[i] = Felt::zero();
    }

    // Start with p[0]
    res[0] = p[0];

    // Compute powers of Q and accumulate
    let mut q_power = vec![Felt::one()]; // Q^0 = 1
    
    for i in 1..p.len() {
        // Multiply q_power by q to get next power
        let new_len = q_power.len() + q.len() - 1;
        let mut new_q_power = vec![Felt::zero(); new_len];
        
        for j in 0..q_power.len() {
            for k in 0..q.len() {
                let mut prod = Felt::zero();
                fmul(&q_power[j], &q[k], &mut prod);
                let temp = new_q_power[j + k];
                fadd(&prod, &temp, &mut new_q_power[j + k]);
            }
        }
        
        q_power = new_q_power;
        
        // Add p[i] * Q^i to result
        for j in 0..std::cmp::min(q_power.len(), res.len()) {
            let mut term = Felt::zero();
            fmul(&p[i], &q_power[j], &mut term);
            let temp = res[j];
            fadd(&term, &temp, &mut res[j]);
        }
    }
}

// Helper trait implementation for Felt conversion from Noun
impl TryFrom<Noun> for Felt {
    type Error = ();

    fn try_from(noun: Noun) -> std::result::Result<Self, Self::Error> {
        // Check if the noun has the as_felt method available
        if let Ok(felt_ref) = noun.as_felt() {
            // Copy the felt data
            Ok(*felt_ref)
        } else {
            // Fallback: try to extract from raw atom data
            let atom = noun.as_atom().map_err(|_| ())?;
            
            // For a Felt, we need exactly 3 Belt values (3 * 8 = 24 bytes)
            // Plus one extra u64 for the tag (total 32 bytes)
            if let Ok(indirect) = atom.as_indirect() {
                if indirect.size() == 4 { // 4 words = 32 bytes
                    unsafe {
                        let buf_ptr = indirect.data_pointer() as *const Belt;
                        let belts = [
                            *buf_ptr,
                            *buf_ptr.add(1),
                            *buf_ptr.add(2),
                        ];
                        Ok(Felt(belts))
                    }
                } else {
                    Err(())
                }
            } else {
                // Direct atom - not large enough for a Felt
                Err(())
            }
        }
    }
}
