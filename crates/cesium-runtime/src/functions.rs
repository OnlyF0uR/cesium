use wasmedge_sdk::{error::CoreError, CallingFrame, Instance, WasmValue};
use wasmedge_sys::AsInstance;

use crate::state::ContractState;

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
    let key_ptr = input[0].to_i32() as u32; // Pointer to the key in memory
    let key_len = input[1].to_i32() as u32; // Length of the key

    // Access the WASM memory to read the key
    let mem = inst.get_memory_mut("memory").unwrap();
    let mem_data = mem.get_data(key_ptr, key_len).unwrap();
    let key = String::from_utf8(mem_data).unwrap();
    // println!("Get state: key = {}", key);

    // Retrieve the value from the `ContractState`
    let return_value = state.storage.get(&key).cloned().unwrap_or_else(Vec::new);
    state.get_state_result = return_value.clone();
    // println!("Get state: value = {:?}", return_value);

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

    let value = state.get_state_result.clone();
    // println!("Write state: value = {:?}", value);

    // Write the cached result to memory at the given offset
    mem.set_data(value.clone(), offset as u32).unwrap();
    state.get_state_result = Vec::new(); // Clear the cached result

    // read from memory iven the offset
    // let mem_data = mem.get_data(offset as u32, value.len() as u32).unwrap();
    // println!("Write state: value = {:?}", mem_data);

    Ok(vec![]) // Return an empty result, as this is a write-only function
}

pub fn h_change_state(
    state: &mut ContractState,
    inst: &mut Instance,
    _caller: &mut CallingFrame,
    input: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    // Verify input parameters
    if input.len() != 4 {
        return Err(CoreError::Execution(
            wasmedge_sdk::error::CoreExecutionError::FuncSigMismatch,
        ));
    }

    // Extract pointers and lengths for key and value
    let key_ptr = input[0].to_i32() as u32; // No change needed here as we are using i32
    let key_len = input[1].to_i32() as u32; // Using i32
    let value_ptr = input[2].to_i32() as u32; // No change needed here as we are using i32
    let value_len = input[3].to_i32() as u32; // Using i32

    // println!(
    //     "Changing state received: key_ptr = {}, key_len = {}, value_ptr = {}, value_len = {}",
    //     key_ptr, key_len, value_ptr, value_len
    // );

    // Access WASM memory and retrieve the key and value
    let mem = inst.get_memory_mut("memory").unwrap();

    // Fetch data using i32 values for lengths
    let key_data = mem.get_data(key_ptr, key_len).unwrap();
    let key = String::from_utf8(key_data).unwrap();

    let value_data = mem.get_data(value_ptr, value_len).unwrap();
    let value = value_data.to_vec(); // Store as Vec<u8>

    state.get_state_result = value.clone();
    state.storage.insert(key, value);

    // let debug_v = state.storage.get("new_key").unwrap();
    // println!("Change state: new_key = {:?}", debug_v);

    Ok(vec![]) // Return the length as a WasmValue
}

pub fn h_commit_state(
    state: &mut ContractState,
    _inst: &mut Instance,
    _caller: &mut CallingFrame,
    _input: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    println!("Commit state: {:?}", state.storage);

    Ok(vec![]) // Return an empty result
}
