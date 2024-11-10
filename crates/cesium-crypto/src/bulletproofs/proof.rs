/// Commitment to a value using a polynomial
#[derive(Clone, Debug)]
pub struct Commitment {
    pub commitment: Vec<u8>,
    pub randomness: Vec<u8>,
}

/// Range proof structure
#[derive(Clone, Debug)]
pub struct RangeProof {
    // Commitment to the bits of the value
    pub a: Vec<u8>,
    // Commitment to the blinding factors
    pub s: Vec<u8>,
    // Vector of L values
    pub l: Vec<Vec<u8>>,
    // Vector of R values
    pub r: Vec<Vec<u8>>,
    // Challenge hash
    pub challenge: Vec<u8>,
}
