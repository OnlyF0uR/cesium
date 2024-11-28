use pqcrypto_sphincsplus::{
    sphincssha2192ssimple, sphincssha2192ssimple_keypair, sphincssha2192ssimple_open,
    sphincssha2192ssimple_sign,
};
use pqcrypto_traits::sign::{PublicKey, SecretKey, SignedMessage};

use crate::errors::CryptoError;

use super::{da::DisplayAddress, PUB_BYTE_LEN, SIG_BYTE_LEN};

pub trait ViewOperations {
    fn pub_key(&self) -> &sphincssha2192ssimple::PublicKey;
    fn verify(&self, msg: &[u8], sig: &[u8]) -> Result<bool, CryptoError> {
        if sig.len() != SIG_BYTE_LEN + msg.len() {
            return Err(CryptoError::InvalidSignature);
        }

        let sm = sphincssha2192ssimple::SignedMessage::from_bytes(sig)?;
        let v_result = sphincssha2192ssimple_open(&sm, self.pub_key());
        if let Err(e) = v_result {
            if e.to_string().contains("verification failed") {
                return Ok(false);
            }

            return Err(CryptoError::UnknownVerificationError);
        }

        Ok(v_result.unwrap() == msg)
    }
    fn get_da(&self) -> DisplayAddress {
        // Get the public key
        let pub_key = self.pub_key().as_bytes();

        // Convert this slice to PublicKeyBytes
        // A test in super::mod.rs will fail if the length of PublicKey does not equal PUB_BYTE_LEN
        // but essentially, we are sure that the length of pub_key is PUB_BYTE_LEN
        // so we can safely use unsafe to cast the slice to PublicKeyBytes
        let id = unsafe { &*(pub_key.as_ptr() as *const [u8; PUB_BYTE_LEN]) };

        // Compute the display address
        let da = DisplayAddress::from_pk(&id);
        da
    }
}

pub struct VerifierPair {
    pub_key: sphincssha2192ssimple::PublicKey,
}

impl VerifierPair {
    #[must_use]
    pub fn new(pub_key: &[u8]) -> Result<Self, CryptoError> {
        let pub_key = sphincssha2192ssimple::PublicKey::from_bytes(pub_key)?;
        Ok(Self { pub_key })
    }

    pub fn from_bytes(pub_key: &[u8]) -> Result<Self, CryptoError> {
        let pub_key = sphincssha2192ssimple::PublicKey::from_bytes(pub_key)?;
        Ok(Self { pub_key })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.pub_key.as_bytes().to_vec()
    }
}

impl ViewOperations for VerifierPair {
    fn pub_key(&self) -> &sphincssha2192ssimple::PublicKey {
        &self.pub_key
    }
}

pub struct SignerPair {
    pub_key: sphincssha2192ssimple::PublicKey,
    sec_key: sphincssha2192ssimple::SecretKey,
}

impl SignerPair {
    pub fn create() -> Self {
        let (pk, sk) = sphincssha2192ssimple_keypair();
        Self {
            pub_key: pk,
            sec_key: sk,
        }
    }

    pub fn sign(&self, msg: &[u8]) -> Vec<u8> {
        let sm = sphincssha2192ssimple_sign(msg, &self.sec_key);
        sm.as_bytes().to_vec()
    }

    pub fn from_bytes(pub_key: &[u8], sec_key: &[u8]) -> Result<Self, CryptoError> {
        let pub_key = sphincssha2192ssimple::PublicKey::from_bytes(pub_key)?;
        let sec_key = sphincssha2192ssimple::SecretKey::from_bytes(sec_key)?;
        Ok(Self { pub_key, sec_key })
    }

    pub fn to_bytes(&self) -> (Vec<u8>, Vec<u8>) {
        (
            self.pub_key.as_bytes().to_vec(),
            self.sec_key.as_bytes().to_vec(),
        )
    }
}

impl ViewOperations for SignerPair {
    fn pub_key(&self) -> &sphincssha2192ssimple::PublicKey {
        &self.pub_key
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signer_sign() {
        let signer = SignerPair::create();
        let msg = b"Hello, World!";
        let sig = signer.sign(msg);

        assert!(signer.verify(msg, &sig).unwrap());
    }

    #[test]
    fn test_verifier_verify() {
        let signer = SignerPair::create();
        let msg = b"Hello, World!";
        let sig = signer.sign(msg);

        let verifier = VerifierPair::new(signer.pub_key().as_bytes()).unwrap();
        assert!(verifier.verify(msg, &sig).unwrap());
    }

    #[test]
    fn test_display_address() {
        let signer = SignerPair::create();
        let da = signer.get_da();

        // Import a verifier from the signer's public key
        let vierifer = VerifierPair::new(signer.pub_key().as_bytes()).unwrap();
        let da2 = vierifer.get_da();

        assert_eq!(da.as_str(), da2.as_str());
    }

    #[test]
    fn test_signer_bytes() {
        let signer = SignerPair::create();
        let (pub_key, sec_key) = signer.to_bytes();

        let signer2 = SignerPair::from_bytes(&pub_key, &sec_key).unwrap();
        assert_eq!(signer.pub_key().as_bytes(), signer2.pub_key().as_bytes());
    }

    #[test]
    fn test_verifier_bytes() {
        let signer = SignerPair::create();
        let verifier = VerifierPair::new(signer.pub_key().as_bytes()).unwrap();
        let pub_key = verifier.to_bytes();

        let verifier2 = VerifierPair::from_bytes(&pub_key).unwrap();
        assert_eq!(
            verifier.pub_key().as_bytes(),
            verifier2.pub_key().as_bytes()
        );
    }
}
