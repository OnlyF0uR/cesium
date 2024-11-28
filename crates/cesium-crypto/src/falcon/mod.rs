// Falcon-1024-padded
pub const PUB_BYTE_LEN: usize = 1793; // 897 (for falcon-512-padded)
pub const SEC_BYTE_LEN: usize = 2305; // 1281 (for falcon-512-padded)
pub const SIG_BYTE_LEN: usize = 1280; // 666 (for falcon-512-padded)

pub type PublicKeyBytes = [u8; PUB_BYTE_LEN];
pub type SecretKeyBytes = [u8; SEC_BYTE_LEN];

pub mod da;
pub mod keypair;
pub mod proofs;

#[cfg(test)]
mod tests {
    use pqcrypto_falcon::falconpadded1024;

    use super::*;

    #[test]
    fn test_lengths() {
        assert_eq!(PUB_BYTE_LEN, falconpadded1024::public_key_bytes());
        assert_eq!(SEC_BYTE_LEN, falconpadded1024::secret_key_bytes());
        assert_eq!(SIG_BYTE_LEN, falconpadded1024::signature_bytes());
    }
}
