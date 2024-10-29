use std::collections::HashMap;

use wasmedge_sdk::{error::CoreError, CallingFrame, Instance, WasmValue};
use wasmedge_sys::AsInstance;

// State types
#[derive(Clone, Debug)]
pub struct ContractState {
    storage: HashMap<String, Vec<u8>>, // Storage for key-value pairs
    get_state_result: Vec<u8>,
}

impl ContractState {
    // Constructor for ContractState
    pub fn new() -> Self {
        Self {
            storage: HashMap::new(), // Initialize storage
            get_state_result: Vec::new(),
        }
    }
}

// Host function to get a value from storage by key
pub fn h_get_state(
    state: &mut ContractState,
    inst: &mut Instance,
    _caller: &mut CallingFrame,
    input: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    // Ensure we have exactly 2 input parameters
    if input.len() != 2 {
        return Err(CoreError::Execution(
            wasmedge_sdk::error::CoreExecutionError::FuncSigMismatch,
        ));
    }

    // Extract pointer and length for the key
    let key_ptr = input[0].to_i32() as usize; // Pointer to the key in memory
    let key_len = input[1].to_i32() as usize; // Length of the key

    // Access the WASM memory to read the key
    let mem = inst.get_memory_mut("memory").unwrap();
    let mem_data = mem.get_data(key_ptr as u32, key_len as u32).unwrap();
    let key = String::from_utf8(mem_data).unwrap();
    println!("Get state: key = {}", key);

    // Retrieve the value from the `ContractState`
    let return_value = state.storage.get(&key).cloned().unwrap_or_else(Vec::new);

    // Return the length of the value
    let return_length = return_value.len() as i32; // Convert length to i32
    Ok(vec![WasmValue::from_i32(return_length)]) // Return the length as a WasmValue
}

pub fn h_write_state_mem(
    state: &mut ContractState,
    inst: &mut Instance,
    _caller: &mut CallingFrame,
    input: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    if input.len() != 1 {
        return Err(CoreError::Execution(
            wasmedge_sdk::error::CoreExecutionError::FuncSigMismatch,
        ));
    }

    // Extract the offset from input
    let offset = input[0].to_i32();

    // Get the WebAssembly memory
    let mut mem = inst.get_memory_mut("memory").unwrap();

    // Write the cached result to memory at the given offset
    mem.set_data(&state.get_state_result, offset as u32)
        .unwrap();

    Ok(vec![]) // Return an empty result, as this is a write-only function
}
