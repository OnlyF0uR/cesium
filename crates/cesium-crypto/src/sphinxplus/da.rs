use rand::RngCore;
use sha3::Digest;

use crate::errors::CryptoError;

use super::PublicKeyBytes;

// For Sphincs+ the public key and display address are interchangeable
// they represent the exact same bytes

pub const DA_BYTE_LEN: usize = 48;
pub type DABytes = [u8; DA_BYTE_LEN];

pub struct DisplayAddress {
    da: [u8; 48],
}

impl DisplayAddress {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let mut da = [0u8; DA_BYTE_LEN];
        rng.fill_bytes(&mut da);

        Self { da }
    }

    pub fn new_from_seed(seed: &[u8]) -> Self {
        let mut hasher = sha3::Sha3_384::new();
        hasher.update(seed);
        let mut da = [0u8; DA_BYTE_LEN];
        da.copy_from_slice(&hasher.finalize());

        Self { da }
    }

    pub fn from_pk(id: &PublicKeyBytes) -> Self {
        Self { da: *id }
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

    pub fn from_str(da: &str) -> Result<Self, bs58::decode::Error> {
        let da = bs58::decode(da).into_vec()?;
        let mut da_bytes = [0u8; DA_BYTE_LEN];
        da_bytes.copy_from_slice(&da);
        Ok(Self { da: da_bytes })
    }
}
