use std::{error::Error, fmt};

#[derive(Debug)]
pub enum GraphError {
    MissingGenesisNode,
    InvalidNodeInput,
    InvalidNodeId,
    ReferenceNodeMismatch,
    MissingSignature,
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
        }
    }
}

// Implement the Error trait for custom error handling
impl Error for GraphError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            GraphError::MissingGenesisNode => None,
            GraphError::InvalidNodeInput => None,
            GraphError::InvalidNodeId => None,
            GraphError::ReferenceNodeMismatch => None,
            GraphError::MissingSignature => None,
        }
    }
}
