use pqcrypto_sphincsplus::sphincsshake192fsimple::{PublicKey, SecretKey};
use pqcrypto_traits::sign::{
    PublicKey as PqPublicKey, SecretKey as PqSecretKey, SignedMessage as PqSignedMessage,
};

use crate::constants::NATIVE_TOKEN;

pub const PUB_BYTE_LEN: usize = 48;
pub const SEC_BYTE_LEN: usize = 96;
pub const SIG_BYTE_LEN: usize = 35664;

pub type PublicKeyBytes = [u8; PUB_BYTE_LEN];
pub type SecretKeyBytes = [u8; SEC_BYTE_LEN];

pub struct KeyPair {
    public_key: PublicKey,
    secret_key: Option<SecretKey>,
}

impl KeyPair {
    pub fn readonly_from_pub(public_key: PublicKey) -> KeyPair {
        KeyPair {
            public_key,
            secret_key: None,
        }
    }

    pub fn readonly_from_readable_pub(public_key_s: &str) -> Result<KeyPair, Box<dyn std::error::Error + Send + Sync>> {
        let pk_bytes = hex::decode(public_key_s)?;
        if pk_bytes.len() != PUB_BYTE_LEN {
            return Err("Invalid public key length".into());
        }
        let public_key = PublicKey::from_bytes(&pk_bytes)?;
        Ok(KeyPair {
            public_key,
            secret_key: None,
        })
    }

    pub fn create() -> KeyPair {
        let (pk, sk) = pqcrypto_sphincsplus::sphincsshake192fsimple_keypair();
        KeyPair {
            public_key: pk,
            secret_key: Some(sk),
        }
    }

    // Idea: Use detached hash instead where the specified buffer,
    // must have the size of the SIG_BYTE_LEN + the message byte length
    // to continue using slices
    pub fn digest(
        &self,
        message: &[u8],
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        if self.secret_key.is_none() {
            return Err("Secret key is missing".into());
        }

        let sm = pqcrypto_sphincsplus::sphincsshake192fsimple_sign(
            message,
            &self.secret_key.as_ref().unwrap(),
        );

        Ok(sm.as_bytes().to_vec())
    }

    pub fn verify(
        &self,
        message: &[u8],
        signature: &[u8],
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let tl = signature.len() - message.len();
        if tl != SIG_BYTE_LEN {
            return Err("Incongruent message and/or signature length".into());
        }

        let sm = PqSignedMessage::from_bytes(signature)?;
        let verified_message =
            pqcrypto_sphincsplus::sphincsshake192fsimple_open(&sm, &self.public_key)?;
        Ok(verified_message == message)
    }

    pub fn secret_key(&self) -> Result<SecretKey, Box<dyn std::error::Error + Send + Sync>> {
        if self.secret_key.is_none() {
            return Err("Secret key is missing".into());
        }
        Ok(self.secret_key.as_ref().unwrap().clone())
    }

    pub fn public_key(&self) -> PublicKey {
        self.public_key.clone()
    }

    pub fn from_bytes(
        public_key_bytes: &[u8],
        secret_key_bytes: &[u8],
    ) -> Result<KeyPair, Box<dyn std::error::Error + Send + Sync>> {
        if public_key_bytes.len() != PUB_BYTE_LEN
        {
            return Err("Invalid public key length".into());
        }
        if secret_key_bytes.len() != SEC_BYTE_LEN
        {
            return Err("Invalid secret key length".into());
        }

        let public_key = PublicKey::from_bytes(public_key_bytes)?;
        let secret_key = SecretKey::from_bytes(secret_key_bytes)?;
        Ok(KeyPair {
            public_key,
            secret_key: Some(secret_key),
        })
    }

    pub fn from_readable(
        public_key_s: &str,
        secret_key_s: &str,
    ) -> Result<KeyPair, Box<dyn std::error::Error + Send + Sync>> {
        let pk_bytes = hex::decode(public_key_s)?;
        let sk_bytes = hex::decode(secret_key_s)?;

        if pk_bytes.len() != PUB_BYTE_LEN {
            return Err("Invalid public key length".into());
        }

        if sk_bytes.len() != SEC_BYTE_LEN {
            return Err("Invalid secret key length".into());
        }

        KeyPair::from_bytes(&pk_bytes, &sk_bytes)
    }

    pub fn to_bytes(
        &self,
    ) -> Result<(&PublicKeyBytes, &SecretKeyBytes), Box<dyn std::error::Error + Send + Sync>> {
        let pk = slice_to_array_48(self.public_key.as_bytes())?;

        if self.secret_key.is_none() {
            return Err("Secret key is missing, use to_public_key_bytes instead".into());
        }

        let sk = slice_to_array_96(self.secret_key.as_ref().unwrap().as_bytes())?;
        Ok((pk, sk))
    }

    pub fn to_readable(
        &self,
    ) -> Result<(String, String), Box<dyn std::error::Error + Send + Sync>> {
        let pk_s = hex::encode(self.public_key.as_bytes());

        if self.secret_key.is_none() {
            return Err("Secret key is missing, use to_public_key_readable instead".into());
        }

        let sk_s = hex::encode(self.secret_key.as_ref().unwrap().as_bytes());
        Ok((pk_s, sk_s))
    }

    pub fn to_public_key_bytes(&self) -> &PublicKeyBytes {
        let bytes = self.public_key.as_bytes();
        match slice_to_array_48(bytes) {
            Ok(formatted) => formatted,
            // Yes there is a panic here, but this should never happen
            // because a public key is only accepted if the length is indeed
            // 48 bytes
            Err(_) => panic!("Invalid public key length"),
        }
    }

    pub fn to_public_key_readable(&self) -> String {
        hex::encode(self.public_key.as_bytes())
    }
}

pub fn address_to_bytes(
    address: &str,
) -> Result<PublicKeyBytes, Box<dyn std::error::Error + Send + Sync>> {
    if address == NATIVE_TOKEN {
        // We can skip the last few bytes
        let mut bytes = NATIVE_TOKEN.as_bytes().to_vec();
        bytes.truncate(PUB_BYTE_LEN);

        let formatted = slice_to_array_48(&bytes.as_slice())?;
        return Ok(formatted.clone());
    }

    let mut buffer = [0u8; 48];
    hex::decode_to_slice(address, &mut buffer)?;

    Ok(buffer)
}
pub fn sig_byte_len(msg_len: usize) -> usize {
   SIG_BYTE_LEN + msg_len
}


fn slice_to_array_48<T>(slice: &[T]) -> Result<&[T; 48], Box<dyn std::error::Error + Send + Sync>> {
    if slice.len() == 48 {
        let ptr = slice.as_ptr() as *const [T; 48];
        unsafe {Ok(&*ptr)}
    } else {
        Err("Invalid slice length".into())
    }
}

fn slice_to_array_96<T>(slice: &[T]) -> Result<&[T; 96], Box<dyn std::error::Error + Send + Sync>> {
    if slice.len() == 96 {
        let ptr = slice.as_ptr() as *const [T; 96];
        unsafe {Ok(&*ptr)}
    } else {
        Err("Invalid slice length".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair() {
        let kp = KeyPair::create();
        let (pk, sk) = kp.to_bytes().unwrap();
        let kp2 = KeyPair::from_bytes(&pk, &sk).unwrap();
        assert_eq!(kp.to_public_key_bytes(), kp2.to_public_key_bytes());
        assert_eq!(kp.to_public_key_readable(), kp2.to_public_key_readable());
    }

    #[test]
    fn test_keypair_readable() {
        let kp = KeyPair::create();
        let (pk_s, sk_s) = kp.to_readable().unwrap();
        let kp2 = KeyPair::from_readable(&pk_s, &sk_s).unwrap();
        assert_eq!(kp.to_public_key_bytes(), kp2.to_public_key_bytes());
        assert_eq!(kp.to_public_key_readable(), kp2.to_public_key_readable());
    }

    #[test]
    fn test_keypair_public_key() {
        let kp = KeyPair::create();

        let pk = kp.to_public_key_bytes();
        let pk2 = PublicKey::from_bytes(&pk).unwrap();
        assert_eq!(kp.public_key.as_bytes(), pk2.as_bytes());
    }

    #[test]
    fn test_keypair_public_key_readable() {
        let kp = KeyPair::create();
        let pk_s = kp.to_public_key_readable();
        let pk2 = PublicKey::from_bytes(&hex::decode(pk_s).unwrap()).unwrap();
        assert_eq!(kp.public_key.as_bytes(), pk2.as_bytes());
    }

    #[test]
    fn test_keypair_secret_key() {
        let kp = KeyPair::create();
        let sk = kp.secret_key.as_ref().unwrap().as_bytes().to_vec();
        let sk2 = SecretKey::from_bytes(&sk).unwrap();
        assert_eq!(kp.secret_key.unwrap().as_bytes(), sk2.as_bytes());
    }

    #[test]
    fn test_keypair_secret_key_readable() {
        let kp = KeyPair::create();
        let sk_s = hex::encode(kp.secret_key.as_ref().unwrap().as_bytes());
        let sk2 = SecretKey::from_bytes(&hex::decode(sk_s).unwrap()).unwrap();
        assert_eq!(kp.secret_key.unwrap().as_bytes(), sk2.as_bytes());
    }

    #[test]
    fn test_keypair_secret_key_missing() {
        let kp = KeyPair {
            public_key: PublicKey::from_bytes(&[0; 48]).unwrap(),
            secret_key: None,
        };
        assert!(kp.to_bytes().is_err());
        assert!(kp.to_readable().is_err());
    }

    #[test]
    fn test_signature() {
        let kp = KeyPair::create();
        let message = b"Hello, world!";
        let sig = kp.digest(message).unwrap();
        assert!(kp.verify(message, &sig).unwrap());
    }
}
