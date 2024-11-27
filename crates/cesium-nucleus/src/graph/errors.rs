use std::{array::TryFromSliceError, fmt, string::FromUtf8Error};

use cesium_nebula::{instruction::InstructionError, transaction::TransactionError};
use cesium_storage::errors::StorageError;

#[derive(Debug)]
pub enum GraphError {
    MissingGenesisNode,
    InvalidNodeInput,
    InvalidNodeId,
    ReferenceNodeMismatch,
    MissingSignature,
    NodeSerializationError(String),
    PutCheckpointError(StorageError),
    TransactionError(TransactionError),
    InstructionError(InstructionError),
    FromUtf8Error(FromUtf8Error),
}

// Implement Display for custom error formatting
impl fmt::Display for GraphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            GraphError::MissingGenesisNode => write!(f, "Missing genesis node"),
            GraphError::InvalidNodeInput => write!(f, "Invalid node input"),
            GraphError::InvalidNodeId => write!(f, "Invalid node id"),
            GraphError::ReferenceNodeMismatch => write!(f, "Reference node mismatch"),
            GraphError::MissingSignature => write!(f, "Missing signature"),
            GraphError::NodeSerializationError(ref e) => {
                write!(f, "Node serialization error: {}", e)
            }
            GraphError::PutCheckpointError(ref e) => write!(f, "Put checkpoint error: {}", e),
            GraphError::TransactionError(ref e) => e.fmt(f),
            GraphError::InstructionError(ref e) => e.fmt(f),
            GraphError::FromUtf8Error(ref e) => e.fmt(f),
        }
    }
}

impl From<TransactionError> for GraphError {
    fn from(err: TransactionError) -> Self {
        GraphError::TransactionError(err)
    }
}

impl From<InstructionError> for GraphError {
    fn from(err: InstructionError) -> Self {
        GraphError::InstructionError(err)
    }
}

impl From<TryFromSliceError> for GraphError {
    fn from(_: TryFromSliceError) -> Self {
        GraphError::InvalidNodeInput
    }
}

impl From<FromUtf8Error> for GraphError {
    fn from(err: FromUtf8Error) -> Self {
        GraphError::FromUtf8Error(err)
    }
}
