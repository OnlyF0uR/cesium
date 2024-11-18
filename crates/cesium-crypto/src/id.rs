use rand::RngCore;
use sha3::Digest;

use crate::keys::{PublicKeyBytes, PUB_BYTE_LEN};

pub fn generate_id() -> PublicKeyBytes {
    let mut rng = rand::thread_rng();
    let mut id = [0u8; PUB_BYTE_LEN];
    rng.fill_bytes(&mut id);
    id
}

pub fn generate_derived_id(id: &[u8]) -> PublicKeyBytes {
    let mut hasher = sha3::Sha3_384::new();
    hasher.update(id);
    hasher
        .finalize()
        .as_slice()
        .try_into()
        .expect("Invalid public key length")
}

pub fn to_readable_id(id: &[u8]) -> String {
    bs58::encode(id).into_string()
}

pub fn from_readable_id(id: &str) -> Result<PublicKeyBytes, bs58::decode::Error> {
    bs58::decode(id)
        .into_vec()
        .map(|v| v.try_into().expect("Invalid public key length"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_id() {
        let id = generate_id();
        assert_eq!(id.len(), PUB_BYTE_LEN);
    }

    #[test]
    fn test_generate_derived_id() {
        let id = generate_id();
        let derived_id = generate_derived_id(&id);
        assert_eq!(derived_id.len(), PUB_BYTE_LEN);
    }

    #[test]
    fn test_readable_id() {
        let id = generate_id();
        let readable_id = to_readable_id(&id);
        let decoded_id = from_readable_id(&readable_id).unwrap();
        assert_eq!(id, decoded_id);
    }
}
