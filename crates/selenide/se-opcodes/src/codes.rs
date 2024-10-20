use crate::errors::OpcodeError;

#[derive(Debug)]
pub enum Opcode {
    // Arithmetic operations
    ADD(u8, u8), // Add two registers (operands are register indices)
    SUB(u8, u8), // Subtract two registers
    MUL(u8, u8), // Multiply two registers
    DIV(u8, u8), // Divide two registers
    MOD(u8, u8), // Modulo two registers
    SQRT(u8),    // Square root of a register
    EXP(u8, u8), // Exponentiation of two registers

    // Memory operations
    LOAD(u8, u8),  // Load value from a register into a local variable
    STORE(u8, u8), // Store a value from a local variable into a register

    // State operations
    SGET(u8, u8), // Loads value from state into register. (e.g. SGET(0, 9) -> From state 0 to register 9)
    SSET(u8, u8), // Push register value into state (e.g. SSET(9, 0) -> From register 9 to state 0)
    SMGET(u8, u8, u8), // Load value from a state map into register (e.g., SMGET(0, 0, 9) -> From state index 0, key index 0 to register 9)
    SMSET(u8, u8, u8), // Push register value into state map (e.g., SMSET(9, 0, 0) -> From register 9 to state index 0, key index 0)

    // Function operations
    CALL(u8), // Call a function by index
    RET,      // Return from a function
}

impl Opcode {
    pub fn from_hex(hex: u8, operands: &[u8]) -> Result<Opcode, OpcodeError> {
        match hex {
            0x01 => {
                if operands.len() != 2 {
                    return Err(OpcodeError::OperandLenghtMismatch(2, operands.len()));
                }
                Ok(Opcode::ADD(operands[0], operands[1]))
            }
            0x02 => {
                if operands.len() != 2 {
                    return Err(OpcodeError::OperandLenghtMismatch(2, operands.len()));
                }
                Ok(Opcode::SUB(operands[0], operands[1]))
            }
            0x03 => {
                if operands.len() != 2 {
                    return Err(OpcodeError::OperandLenghtMismatch(2, operands.len()));
                }
                Ok(Opcode::MUL(operands[0], operands[1]))
            }
            0x04 => {
                if operands.len() != 2 {
                    return Err(OpcodeError::OperandLenghtMismatch(2, operands.len()));
                }
                Ok(Opcode::DIV(operands[0], operands[1]))
            }
            0x05 => {
                if operands.len() != 2 {
                    return Err(OpcodeError::OperandLenghtMismatch(2, operands.len()));
                }
                Ok(Opcode::MOD(operands[0], operands[1]))
            }
            0x06 => {
                if operands.len() != 1 {
                    return Err(OpcodeError::OperandLenghtMismatch(1, operands.len()));
                }
                Ok(Opcode::SQRT(operands[0]))
            }
            0x07 => {
                if operands.len() != 2 {
                    return Err(OpcodeError::OperandLenghtMismatch(2, operands.len()));
                }
                Ok(Opcode::EXP(operands[0], operands[1]))
            }
            0x08 => {
                if operands.len() != 2 {
                    return Err(OpcodeError::OperandLenghtMismatch(2, operands.len()));
                }
                Ok(Opcode::LOAD(operands[0], operands[1]))
            }
            0x09 => {
                if operands.len() != 2 {
                    return Err(OpcodeError::OperandLenghtMismatch(2, operands.len()));
                }
                Ok(Opcode::STORE(operands[0], operands[1]))
            }
            0x0A => {
                if operands.len() != 2 {
                    return Err(OpcodeError::OperandLenghtMismatch(2, operands.len()));
                }
                Ok(Opcode::SGET(operands[0], operands[1]))
            }
            0x0B => {
                if operands.len() != 2 {
                    return Err(OpcodeError::OperandLenghtMismatch(2, operands.len()));
                }
                Ok(Opcode::SSET(operands[0], operands[1]))
            }
            0x0C => {
                if operands.len() != 3 {
                    return Err(OpcodeError::OperandLenghtMismatch(3, operands.len()));
                }
                Ok(Opcode::SMGET(operands[0], operands[1], operands[2]))
            }
            0x0D => {
                if operands.len() != 3 {
                    return Err(OpcodeError::OperandLenghtMismatch(3, operands.len()));
                }
                Ok(Opcode::SMSET(operands[0], operands[1], operands[2]))
            }
            0x0E => {
                if operands.len() != 1 {
                    return Err(OpcodeError::OperandLenghtMismatch(1, operands.len()));
                }
                Ok(Opcode::CALL(operands[0]))
            }
            0x0F => Ok(Opcode::RET),

            _ => Err(OpcodeError::InvalidOpcode(hex)),
        }
    }

    pub fn to_hex(&self) -> u8 {
        match self {
            Opcode::ADD(_, _) => 0x01,
            Opcode::SUB(_, _) => 0x02,
            Opcode::MUL(_, _) => 0x03,
            Opcode::DIV(_, _) => 0x04,
            Opcode::MOD(_, _) => 0x05,
            Opcode::SQRT(_) => 0x06,
            Opcode::EXP(_, _) => 0x07,
            Opcode::LOAD(_, _) => 0x08,
            Opcode::STORE(_, _) => 0x09,
            Opcode::SGET(_, _) => 0x0A,
            Opcode::SSET(_, _) => 0x0B,
            Opcode::SMGET(_, _, _) => 0x0C,
            Opcode::SMSET(_, _, _) => 0x0D,
            Opcode::CALL(_) => 0x0E,
            Opcode::RET => 0x0F,
        }
    }
}
