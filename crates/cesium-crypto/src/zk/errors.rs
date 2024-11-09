use std::{error::Error, fmt};

#[derive(Debug)]
pub enum ZkError {
    InvalidSecret,
    InvalidCommitment,
    InvalidResponse,
    VerificationError(String),
    KeyGenerationError,
    SigningError(String),
}

impl fmt::Display for ZkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            ZkError::InvalidSecret => write!(f, "Invalid secret"),
            ZkError::InvalidCommitment => write!(f, "Invalid commitment"),
            ZkError::InvalidResponse => write!(f, "Invalid response"),
            ZkError::VerificationError(ref e) => {
                write!(f, "Signature verification error: {}", e)
            }
            ZkError::KeyGenerationError => write!(f, "Key generation error"),
            ZkError::SigningError(ref e) => write!(f, "Signing error: {}", e),
        }
    }
}

impl Error for ZkError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            ZkError::InvalidSecret => None,
            ZkError::InvalidCommitment => None,
            ZkError::InvalidResponse => None,
            ZkError::VerificationError(_) => None,
            ZkError::KeyGenerationError => None,
            ZkError::SigningError(_) => None,
        }
    }
}
