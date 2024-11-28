use cesium_crypto::errors::CryptoError;
use tokio::task::JoinError;

#[derive(Debug)]
pub enum InstructionError {
    NoInstructions,
    InvalidInstructionType,
    InstructionLengthIncongruency,
    ByteMismatch,
    InsufficientFunds,
    OutOfGas,
    CryptoError(CryptoError),
    JoinError(JoinError),
}

impl std::fmt::Display for InstructionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstructionError::NoInstructions => write!(f, "Transaction has no instructions"),
            InstructionError::InvalidInstructionType => write!(f, "Invalid instruction type"),
            InstructionError::InstructionLengthIncongruency => {
                write!(f, "Instruction length incongruency")
            }
            InstructionError::ByteMismatch => write!(f, "Byte mismatch"),
            InstructionError::InsufficientFunds => write!(f, "Insufficient funds"),
            InstructionError::OutOfGas => write!(f, "Out of gas"),
            InstructionError::CryptoError(e) => e.fmt(f),
            InstructionError::JoinError(e) => e.fmt(f),
        }
    }
}

impl From<CryptoError> for InstructionError {
    fn from(e: CryptoError) -> Self {
        InstructionError::CryptoError(e)
    }
}

impl From<JoinError> for InstructionError {
    fn from(e: JoinError) -> Self {
        InstructionError::JoinError(e)
    }
}

impl std::error::Error for InstructionError {}
