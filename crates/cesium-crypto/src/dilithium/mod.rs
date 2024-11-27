// Dilihtium3
pub const PUB_BYTE_LEN: usize = 1312;
pub const SEC_BYTE_LEN: usize = 2560;
pub const SIG_BYTE_LEN: usize = 2420;

pub type PublicKeyBytes = [u8; PUB_BYTE_LEN];
pub type SecretKeyBytes = [u8; SEC_BYTE_LEN];

pub mod da;
pub mod keypair;
pub mod proofs;

#[cfg(test)]
mod tests {
    use pqcrypto_mldsa::mldsa44;

    use super::*;

    #[test]
    fn test_lengths() {
        assert_eq!(PUB_BYTE_LEN, mldsa44::public_key_bytes());
        assert_eq!(SEC_BYTE_LEN, mldsa44::secret_key_bytes());
        assert_eq!(SIG_BYTE_LEN, mldsa44::signature_bytes());
    }
}
