use wasmedge_sdk::{error::CoreError, CallingFrame, Instance, ValType, WasmValue};

// State types
#[derive(Clone, Debug)]
pub struct ContractState {
    storage: Vec<i64>, // Storage for key-value pairs
}

impl ContractState {
    // Constructor for ContractState
    pub fn new() -> Self {
        Self {
            storage: Vec::new(), // Initialize storage
        }
    }
}

// Host function to get a value from storage by key
pub fn get_state(
    state: &mut ContractState, // State modification, write operation
    _inst: &mut Instance,
    _caller: &mut CallingFrame,
    input: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    // Check input parameters
    if input.len() != 1 {
        return Err(CoreError::Execution(
            wasmedge_sdk::error::CoreExecutionError::FuncSigMismatch,
        ));
    }

    let state_index_value = if let ValType::I32 = input[0].ty() {
        input[0].to_i32()
    } else {
        return Err(CoreError::Execution(
            wasmedge_sdk::error::CoreExecutionError::FuncSigMismatch,
        ));
    };

    if state_index_value < 0 || state_index_value as usize >= state.storage.len() {
        return Err(CoreError::Execution(
            wasmedge_sdk::error::CoreExecutionError::FuncSigMismatch,
        ));
    }

    let value = state.storage[state_index_value as usize] as i64;
    let return_value = WasmValue::from_i64(value);

    Ok(vec![return_value])
}

pub fn update_state(
    state: &mut ContractState, // State modification, write operation
    _inst: &mut Instance,
    _caller: &mut CallingFrame,
    input: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    // Check input parameters
    if input.len() != 2 {
        return Err(CoreError::Execution(
            wasmedge_sdk::error::CoreExecutionError::FuncSigMismatch,
        ));
    }

    let state_index_value = if let ValType::I32 = input[0].ty() {
        input[0].to_i32()
    } else {
        return Err(CoreError::Execution(
            wasmedge_sdk::error::CoreExecutionError::FuncSigMismatch,
        ));
    };

    if state_index_value < 0 || state_index_value as usize >= state.storage.len() {
        return Err(CoreError::Execution(
            wasmedge_sdk::error::CoreExecutionError::FuncSigMismatch,
        ));
    }

    let value = if let ValType::I64 = input[1].ty() {
        input[1].to_i64()
    } else {
        return Err(CoreError::Execution(
            wasmedge_sdk::error::CoreExecutionError::FuncSigMismatch,
        ));
    };

    state.storage[state_index_value as usize] = value;

    Ok(vec![])
}
