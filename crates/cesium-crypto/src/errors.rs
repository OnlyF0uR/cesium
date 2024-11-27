#[derive(Debug)]
pub enum CryptoError {
    PQCryptoError(pqcrypto_traits::Error),
    UnknownVerificationError,
    BaseEncodeError(bs58::encode::Error),
    BaseDecodeError(bs58::decode::Error),
    InvalidSignature,
    InvalidDisplayAddress,
    ZkInvalidSecret,
    ZkInvalidCommitment,
    ZkInvalidResponse,
    ZkKeyGenerationError,
}

impl std::fmt::Display for CryptoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CryptoError::PQCryptoError(e) => e.fmt(f),
            CryptoError::UnknownVerificationError => write!(f, "Unknown verification error"),
            CryptoError::BaseEncodeError(e) => e.fmt(f),
            CryptoError::BaseDecodeError(e) => e.fmt(f),
            CryptoError::InvalidSignature => write!(f, "Invalid signature"),
            CryptoError::InvalidDisplayAddress => write!(f, "Invalid display address"),
            CryptoError::ZkInvalidSecret => write!(f, "Invalid secret"),
            CryptoError::ZkInvalidCommitment => write!(f, "Invalid commitment"),
            CryptoError::ZkInvalidResponse => write!(f, "Invalid response"),
            CryptoError::ZkKeyGenerationError => write!(f, "Key generation error"),
        }
    }
}

impl From<bs58::encode::Error> for CryptoError {
    fn from(e: bs58::encode::Error) -> Self {
        CryptoError::BaseEncodeError(e)
    }
}

impl From<bs58::decode::Error> for CryptoError {
    fn from(e: bs58::decode::Error) -> Self {
        CryptoError::BaseDecodeError(e)
    }
}

impl From<pqcrypto_traits::Error> for CryptoError {
    fn from(e: pqcrypto_traits::Error) -> Self {
        CryptoError::PQCryptoError(e)
    }
}

// impl std::error::Error for CryptoError {}
