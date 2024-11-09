use cesium_standards::StandardToken;
use pqcrypto_sphincsplus::sphincsshake192ssimple::{PublicKey, SecretKey};
use pqcrypto_traits::sign::{
    PublicKey as PqPublicKey, SecretKey as PqSecretKey, SignedMessage as PqSignedMessage,
};

pub const PUB_BYTE_LEN: usize = 48;
pub const SEC_BYTE_LEN: usize = 96;
pub const SIG_BYTE_LEN: usize = 16224; // 35664 (192f), 16224 (192s)

pub type PublicKeyBytes = [u8; PUB_BYTE_LEN];
pub type SecretKeyBytes = [u8; SEC_BYTE_LEN];

pub struct Account {
    public_key: PublicKey,
    secret_key: Option<SecretKey>,
}

macro_rules! ensure_secret_key {
    ($self:expr) => {
        match $self.secret_key.as_ref() {
            Some(sk) => sk,
            None => return Err("Secret key is missing".into()),
        }
    };
}

impl Account {
    pub fn readonly_from_pub(public_key: PublicKey) -> Self {
        Self {
            public_key,
            secret_key: None,
        }
    }

    pub fn readonly_from_readable_pub(
        public_key_s: &str,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let pk_bytes = bs58::decode(public_key_s).into_vec()?;
        if pk_bytes.len() != PUB_BYTE_LEN {
            return Err("Invalid public key length".into());
        }
        let public_key = PublicKey::from_bytes(&pk_bytes)?;
        Ok(Self::readonly_from_pub(public_key))
    }

    pub fn create() -> Self {
        let (pk, sk) = pqcrypto_sphincsplus::sphincsshake192ssimple_keypair();
        Self {
            public_key: pk,
            secret_key: Some(sk),
        }
    }

    pub fn digest(
        &self,
        message: &[u8],
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let sk = ensure_secret_key!(self);
        let sm = pqcrypto_sphincsplus::sphincsshake192ssimple_sign(message, sk);
        Ok(sm.as_bytes().to_vec())
    }

    pub fn verify(
        &self,
        message: &[u8],
        signature: &[u8],
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        if signature.len() < message.len() + SIG_BYTE_LEN {
            return Err("Invalid signature length".into());
        }

        let sm = PqSignedMessage::from_bytes(signature)?;
        let verified_message =
            pqcrypto_sphincsplus::sphincsshake192ssimple_open(&sm, &self.public_key)?;
        Ok(verified_message == message)
    }

    pub fn secret_key(&self) -> Result<&SecretKey, Box<dyn std::error::Error + Send + Sync>> {
        Ok(ensure_secret_key!(self))
    }

    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }

    pub fn from_bytes(
        public_key_bytes: &[u8],
        secret_key_bytes: &[u8],
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        if public_key_bytes.len() != PUB_BYTE_LEN || secret_key_bytes.len() != SEC_BYTE_LEN {
            return Err("Invalid key length".into());
        }

        let public_key = PublicKey::from_bytes(public_key_bytes)?;
        let secret_key = SecretKey::from_bytes(secret_key_bytes)?;
        Ok(Self {
            public_key,
            secret_key: Some(secret_key),
        })
    }

    pub fn from_readable(
        public_key_s: &str,
        secret_key_s: &str,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let pk_bytes = bs58::decode(public_key_s).into_vec()?;
        let sk_bytes = hex::decode(secret_key_s)?;

        Self::from_bytes(&pk_bytes, &sk_bytes)
    }

    pub fn to_bytes(
        &self,
    ) -> Result<(&PublicKeyBytes, &SecretKeyBytes), Box<dyn std::error::Error + Send + Sync>> {
        let pk = slice_to_array_48(self.public_key.as_bytes())?;
        let sk = slice_to_array_96(ensure_secret_key!(self).as_bytes())?;
        Ok((pk, sk))
    }

    pub fn to_readable(
        &self,
    ) -> Result<(String, String), Box<dyn std::error::Error + Send + Sync>> {
        let pk_s = bs58::encode(self.public_key.as_bytes()).into_string();
        let sk_s = hex::encode(ensure_secret_key!(self).as_bytes());
        Ok((pk_s, sk_s))
    }

    pub fn to_public_key_bytes(&self) -> &PublicKeyBytes {
        slice_to_array_48(self.public_key.as_bytes())
            .expect("Invalid public key length; expected 48 bytes")
    }

    pub fn to_public_key_readable(&self) -> String {
        bs58::encode(self.public_key.as_bytes()).into_string()
    }
}

pub fn address_to_bytes(
    address: &str,
) -> Result<PublicKeyBytes, Box<dyn std::error::Error + Send + Sync>> {
    let bytes = if StandardToken::is_standard_token(&address) {
        let mut standard_bytes = address.as_bytes().to_vec();
        standard_bytes.truncate(PUB_BYTE_LEN);
        standard_bytes
    } else {
        bs58::decode(address).into_vec()?
    };
    slice_to_array_48(&bytes).map(|arr| *arr)
}

pub fn sig_byte_len(msg_len: usize) -> usize {
    SIG_BYTE_LEN + msg_len
}

pub fn slice_to_array_48<T>(
    slice: &[T],
) -> Result<&[T; 48], Box<dyn std::error::Error + Send + Sync>> {
    if slice.len() == 48 {
        Ok(unsafe { &*(slice.as_ptr() as *const [T; 48]) })
    } else {
        Err("Invalid slice length".into())
    }
}

pub fn slice_to_array_96<T>(
    slice: &[T],
) -> Result<&[T; 96], Box<dyn std::error::Error + Send + Sync>> {
    if slice.len() == 96 {
        Ok(unsafe { &*(slice.as_ptr() as *const [T; 96]) })
    } else {
        Err("Invalid slice length".into())
    }
}

#[cfg(test)]
mod tests {
    use cesium_standards::NATIVE_TOKEN;

    use super::*;

    #[test]
    fn test_account() {
        let kp = Account::create();
        let (pk, sk) = kp.to_bytes().unwrap();
        let kp2 = Account::from_bytes(pk, sk).unwrap();
        assert_eq!(kp.to_public_key_bytes(), kp2.to_public_key_bytes());
        assert_eq!(kp.to_public_key_readable(), kp2.to_public_key_readable());
    }

    #[test]
    fn test_account_readable() {
        let kp = Account::create();
        let (pk_s, sk_s) = kp.to_readable().unwrap();
        let kp2 = Account::from_readable(&pk_s, &sk_s).unwrap();

        assert_eq!(kp.to_public_key_bytes(), kp2.to_public_key_bytes());
        assert_eq!(kp.to_public_key_readable(), kp2.to_public_key_readable());
    }

    #[test]
    fn test_account_public_key() {
        let kp = Account::create();

        let pk = kp.to_public_key_bytes();
        let pk2 = PublicKey::from_bytes(pk).unwrap();
        assert_eq!(kp.public_key.as_bytes(), pk2.as_bytes());
    }

    #[test]
    fn test_account_public_key_readable() {
        // Create a new Account
        let kp = Account::create();
        // Get a readable public key (wallet address)
        let pk_s = kp.to_public_key_readable();

        // Convert this readable back to bytes
        let pk2 = address_to_bytes(&pk_s).unwrap();

        // Compare to the bytes of the original key
        assert_eq!(kp.public_key.as_bytes(), pk2);
    }

    #[test]
    fn test_account_secret_key() {
        let kp = Account::create();
        let sk = kp.secret_key.as_ref().unwrap().as_bytes().to_vec();
        let sk2 = SecretKey::from_bytes(&sk).unwrap();
        assert_eq!(kp.secret_key.unwrap().as_bytes(), sk2.as_bytes());
    }

    #[test]
    fn test_account_secret_key_readable() {
        let kp = Account::create();
        let sk_s = hex::encode(kp.secret_key.as_ref().unwrap().as_bytes());
        let sk2 = SecretKey::from_bytes(&hex::decode(sk_s).unwrap()).unwrap();
        assert_eq!(kp.secret_key.unwrap().as_bytes(), sk2.as_bytes());
    }

    #[test]
    fn test_account_secret_key_missing() {
        let kp = Account {
            public_key: PublicKey::from_bytes(&[0; 48]).unwrap(),
            secret_key: None,
        };
        assert!(kp.to_bytes().is_err());
        assert!(kp.to_readable().is_err());
    }

    #[test]
    fn test_signature() {
        let kp = Account::create();
        let message = b"Hello, world!";
        let sig = kp.digest(message).unwrap();
        assert!(kp.verify(message, &sig).unwrap());
    }

    #[test]
    fn test_address_to_bytes_native() {
        let address = NATIVE_TOKEN;
        let bytes = address_to_bytes(address).unwrap();

        let mut expected = NATIVE_TOKEN.as_bytes().to_vec();
        expected.truncate(PUB_BYTE_LEN);

        assert_eq!(bytes, expected.as_slice());
    }
}
