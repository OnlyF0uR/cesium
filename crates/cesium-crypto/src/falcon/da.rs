use rand::RngCore;
use sha3::Digest;

use crate::errors::CryptoError;

use super::{PublicKeyBytes, PUB_BYTE_LEN};

// For Dilithium the public key and display address are not interchangeable
// the display address is a hash of the public key, it is thus not reversible
// and is merely used to identify the public key

pub const DA_BYTE_LEN: usize = 32;
pub type DABytes = [u8; DA_BYTE_LEN];

pub struct DisplayAddress {
    da: [u8; DA_BYTE_LEN],
}

impl DisplayAddress {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let mut da = [0u8; DA_BYTE_LEN];
        rng.fill_bytes(&mut da);

        Self { da }
    }

    pub fn new_from_seed(seed: &[u8]) -> Self {
        let nonce = rand::thread_rng().next_u64();

        let mut hasher = sha3::Sha3_256::new();
        hasher.update(seed);
        hasher.update(nonce.to_le_bytes());

        let da = hasher.finalize().as_slice().try_into().unwrap();
        Self { da }
    }

    pub fn try_from_pk(id: &[u8]) -> Result<Self, CryptoError> {
        if id.len() != PUB_BYTE_LEN {
            return Err(CryptoError::InvalidDisplayAddress);
        }

        let mut hasher = sha3::Sha3_256::new();
        hasher.update(id);
        let mut da = [0u8; DA_BYTE_LEN];
        da.copy_from_slice(&hasher.finalize());

        Ok(Self { da })
    }

    pub fn from_pk(id: &PublicKeyBytes) -> Self {
        let mut hasher = sha3::Sha3_256::new();
        hasher.update(id);
        let mut da = [0u8; DA_BYTE_LEN];
        da.copy_from_slice(&hasher.finalize());

        Self { da }
    }

    pub fn as_bytes(&self) -> &DABytes {
        &self.da
    }

    pub fn from_bytes(da: &[u8]) -> Result<Self, CryptoError> {
        if da.len() != DA_BYTE_LEN {
            return Err(CryptoError::InvalidDisplayAddress);
        }

        Ok(Self {
            da: unsafe { *(da.as_ptr() as *const [u8; DA_BYTE_LEN]) },
        })
    }

    pub fn as_str(&self) -> String {
        bs58::encode(&self.da).into_string()
    }
}
