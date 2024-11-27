// Dilihtium3
pub const PUB_BYTE_LEN: usize = 48;
pub const SEC_BYTE_LEN: usize = 96;
pub const SIG_BYTE_LEN: usize = 16224; // 35664 (192f), 16224 (192s)

pub type PublicKeyBytes = [u8; PUB_BYTE_LEN];
pub type SecretKeyBytes = [u8; SEC_BYTE_LEN];

pub mod da;
pub mod keypair;
pub mod proofs;

#[cfg(test)]
mod tests {
    use pqcrypto_sphincsplus::{
        sphincssha2192ssimple_public_key_bytes, sphincssha2192ssimple_secret_key_bytes,
        sphincssha2192ssimple_signature_bytes,
    };

    use super::*;

    #[test]
    fn test_lengths() {
        assert_eq!(PUB_BYTE_LEN, sphincssha2192ssimple_public_key_bytes());
        assert_eq!(SEC_BYTE_LEN, sphincssha2192ssimple_secret_key_bytes());
        assert_eq!(SIG_BYTE_LEN, sphincssha2192ssimple_signature_bytes());
    }
}
