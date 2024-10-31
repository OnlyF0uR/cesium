use rand::RngCore;
use sha3::Digest;

use crate::keys::PUB_BYTE_LEN;

pub fn generate_id() -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let mut id = [0u8; 32];
    rng.fill_bytes(&mut id);

    let mut hasher = sha3::Sha3_384::new();
    hasher.update(id);
    let result = hasher.finalize().to_vec();

    // We want to ensure that our hash is the size of
    // a public key, so we can use it as an ID, now a
    // sha3_384 hash is 48 bytes, so that works.
    if result.len() != PUB_BYTE_LEN {
        panic!("Invalid public key length");
    }

    result
}

pub fn to_readable_id(id: &[u8]) -> String {
    bs58::encode(id).into_string()
}
