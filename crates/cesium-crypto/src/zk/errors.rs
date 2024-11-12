use std::{error::Error, fmt};

use crate::keys::AccountError;

#[derive(Debug)]
pub enum ZkError {
    InvalidSecret,
    InvalidCommitment,
    InvalidResponse,
    AccountError(AccountError),
    KeyGenerationError,
    SigningError(String),
}

impl fmt::Display for ZkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            ZkError::InvalidSecret => write!(f, "Invalid secret"),
            ZkError::InvalidCommitment => write!(f, "Invalid commitment"),
            ZkError::InvalidResponse => write!(f, "Invalid response"),
            ZkError::AccountError(ref e) => write!(f, "Account error: {}", e),
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
            ZkError::AccountError(ref e) => Some(e),
            ZkError::KeyGenerationError => None,
            ZkError::SigningError(_) => None,
        }
    }
}
