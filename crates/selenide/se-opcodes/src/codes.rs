#[derive(Debug)]
pub enum Opcode {
    // Arithmetic operations
    ADD(u8, u8), // Add two registers (operands are register indices)
    SUB(u8, u8), // Subtract two registers
    MUL(u8, u8), // Multiply two registers
    DIV(u8, u8), // Divide two registers
    MOD(u8, u8), // Modulo two registers
    SQRT(u8),    // Square root of a register

    // Memory operations
    LOAD(u8, u8),  // Load value from a register into a local variable
    STORE(u8, u8), // Store a value from a local variable into a register

    // State operations
    SGET(String, u8), // Loads a value from a state variable into a register (e.g., SGET("counter", 0))
    SSET(u8, String), // Store a register value into a state variable (e.g., SSET(1, "counter"))
    SMGET(String, String, u8), // Load a value from a state map into a register (e.g., SMGET("balances", "user1", 0))
    SMSET(u8, String, String), // Store a register value into a state map (e.g., SMSET(1, "balances", "user1"))

    // Function operations
    CALL(u8), // Call a function by index
    RET,      // Return from a function
}
