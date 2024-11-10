use rand::{thread_rng, RngCore};
use sha3::{Digest, Sha3_384};

use super::{
    fields::FieldElement,
    proof::{Commitment, RangeProof},
};

/// Prover for range proofs
pub struct Prover {
    value: u64,
    bit_length: usize,
}

impl Prover {
    pub fn new(value: u64, bit_length: usize) -> Self {
        Self { value, bit_length }
    }

    pub fn prove(&self) -> RangeProof {
        let mut rng = thread_rng();

        // Convert value to binary
        let bits: Vec<bool> = (0..self.bit_length)
            .map(|i| ((self.value >> i) & 1) == 1)
            .collect();

        // Generate randomness for commitments
        let mut a_rand = vec![0u8; 32];
        rng.fill_bytes(&mut a_rand);

        let mut s_rand = vec![0u8; 32];
        rng.fill_bytes(&mut s_rand);

        // Commit to bits
        let mut a_value = FieldElement::new(0);
        for (i, &bit) in bits.iter().enumerate() {
            if bit {
                a_value = a_value.add(&FieldElement::new(1 << i));
            }
        }
        let a_comm = self.commit(&a_value, &a_rand);

        // Generate mask commitment
        let s_value = FieldElement::random(&mut rng);
        let s_comm = self.commit(&s_value, &s_rand);

        // Generate l_val and r_val values for the inner product argument
        let mut l_val = Vec::new();
        let mut r_val = Vec::new();

        // For each round of the protocol
        for _ in 0..self.bit_length.trailing_zeros() {
            let mut l_rand = vec![0u8; 32];
            rng.fill_bytes(&mut l_rand);
            let mut r_rand = vec![0u8; 32];
            rng.fill_bytes(&mut r_rand);

            let l_value = FieldElement::random(&mut rng);
            let r_value = FieldElement::random(&mut rng);

            let l_comm = self.commit(&l_value, &l_rand);
            let r_comm = self.commit(&r_value, &r_rand);

            l_val.push(l_comm.commitment);
            r_val.push(r_comm.commitment);
        }

        // Generate challenge
        let mut hasher = Sha3_384::new();
        hasher.update(&a_comm.commitment);
        hasher.update(&s_comm.commitment);
        for l in &l_val {
            hasher.update(l);
        }
        for r in &r_val {
            hasher.update(r);
        }

        RangeProof {
            a: a_comm.commitment,
            s: s_comm.commitment,
            l: l_val,
            r: r_val,
            challenge: hasher.finalize().to_vec(),
        }
    }

    fn commit(&self, value: &FieldElement, randomness: &[u8]) -> Commitment {
        let mut hasher = Sha3_384::new();
        hasher.update(value.value.to_le_bytes());
        hasher.update(randomness);

        Commitment {
            commitment: hasher.finalize().to_vec(),
            randomness: randomness.to_vec(),
        }
    }
}
