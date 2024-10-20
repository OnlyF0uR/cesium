use std::{error::Error, fmt};

#[derive(Debug)]
pub enum OpcodeError {
    InvalidOpcode(u8),
    InvalidOperand(u8),
    OperandLenghtMismatch(usize, usize),
}

impl fmt::Display for OpcodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            OpcodeError::InvalidOpcode(opcode) => write!(f, "Invalid opcode: {}", opcode),
            OpcodeError::InvalidOperand(operand) => write!(f, "Invalid operand: {}", operand),
            OpcodeError::OperandLenghtMismatch(expected, actual) => {
                write!(
                    f,
                    "Operand length mismatch: expected {}, got {}",
                    expected, actual
                )
            }
        }
    }
}

impl Error for OpcodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            OpcodeError::InvalidOpcode(_) => None,
            OpcodeError::InvalidOperand(_) => None,
            OpcodeError::OperandLenghtMismatch(_, _) => None,
        }
    }
}

#[derive(Debug)]
pub enum RegistryError {
    InvalidStateRegister(String),
    InvalidLocalRegister(String),
    TypeMismatch(String, String, String),
    OutOfBounds(usize, usize),
}

impl fmt::Display for RegistryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RegistryError::InvalidStateRegister(ref register) => {
                write!(f, "Invalid state register: {}", register)
            }
            RegistryError::InvalidLocalRegister(ref register) => {
                write!(f, "Invalid local register: {}", register)
            }
            RegistryError::TypeMismatch(ref register, ref expected, ref actual) => {
                write!(
                    f,
                    "Type mismatch in register {}: expected {}, got {}",
                    register, expected, actual
                )
            }
            RegistryError::OutOfBounds(ref index, ref size) => {
                write!(f, "Index out of bounds: {} (size: {})", index, size)
            }
        }
    }
}

impl Error for RegistryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            RegistryError::InvalidStateRegister(_) => None,
            RegistryError::InvalidLocalRegister(_) => None,
            RegistryError::TypeMismatch(_, _, _) => None,
            RegistryError::OutOfBounds(_, _) => None,
        }
    }
}
