use std::fmt;

#[derive(Debug)]
pub enum RuntimeError {
    WasmerInstantiationError(wasmer::InstantiationError),
    WasmerCompileError(wasmer::CompileError),
    WasmerExportError(wasmer::ExportError),
    WasmerRuntimeError(wasmer::RuntimeError),
    WasmerMemoryAccessError(wasmer::MemoryAccessError),
    MemoryNotInitialized,
    MemoryOutOfBounds,
    MemoryAllocationError,
    InvalidExportReturnType,
    OutOfGas,
}

// Implement Display for custom error formatting
impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            RuntimeError::WasmerInstantiationError(ref e) => {
                write!(f, "Wasmer instantiation error: {}", e)
            }
            RuntimeError::WasmerExportError(ref e) => write!(f, "Wasmer export error: {}", e),
            RuntimeError::WasmerCompileError(ref e) => write!(f, "Wasmer compile error: {}", e),
            RuntimeError::WasmerRuntimeError(ref e) => write!(f, "Wasmer runtime error: {}", e),
            RuntimeError::WasmerMemoryAccessError(ref e) => {
                write!(f, "Wasmer memory access error: {}", e)
            }
            RuntimeError::MemoryNotInitialized => write!(f, "Memory not initialized"),
            RuntimeError::MemoryOutOfBounds => write!(f, "Memory out of bounds"),
            RuntimeError::MemoryAllocationError => write!(f, "Memory allocation error"),
            RuntimeError::InvalidExportReturnType => write!(f, "Invalid export return type"),
            RuntimeError::OutOfGas => write!(f, "Out of gas"),
        }
    }
}

impl std::error::Error for RuntimeError {}

impl From<wasmer::InstantiationError> for RuntimeError {
    fn from(error: wasmer::InstantiationError) -> Self {
        RuntimeError::WasmerInstantiationError(error)
    }
}

impl From<wasmer::CompileError> for RuntimeError {
    fn from(error: wasmer::CompileError) -> Self {
        RuntimeError::WasmerCompileError(error)
    }
}

impl From<wasmer::ExportError> for RuntimeError {
    fn from(error: wasmer::ExportError) -> Self {
        RuntimeError::WasmerExportError(error)
    }
}

impl From<wasmer::RuntimeError> for RuntimeError {
    fn from(error: wasmer::RuntimeError) -> Self {
        RuntimeError::WasmerRuntimeError(error)
    }
}

impl From<wasmer::MemoryAccessError> for RuntimeError {
    fn from(error: wasmer::MemoryAccessError) -> Self {
        RuntimeError::WasmerMemoryAccessError(error)
    }
}
