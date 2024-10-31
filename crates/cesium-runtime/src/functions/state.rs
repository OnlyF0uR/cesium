use wasmedge_sdk::{error::CoreError, CallingFrame, Instance, WasmValue};
use wasmedge_sys::AsInstance;

use crate::{
    convert::wasm_encoder,
    data::{save_state, MAX_MEMORY_OFFSET},
    env::ContractEnv,
};

pub fn h_define_state(
    env: &mut ContractEnv,
    _inst: &mut Instance,
    _caller: &mut CallingFrame,
    input: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    // We get one input, which is the length of the storage array
    if input.len() != 1 {
        return Err(CoreError::Execution(
            wasmedge_sdk::error::CoreExecutionError::FuncSigMismatch,
        ));
    }

    if env.state.initialized {
        return Err(CoreError::Execution(
            wasmedge_sdk::error::CoreExecutionError::FuncSigMismatch,
        ));
    }

    env.state.initialized = true;

    let storage_len = input[0].to_i32() as usize;
    env.state.data = vec![Vec::new(); storage_len];

    Ok(wasm_encoder::empty_value())
}

// Host function to get a value from storage by key
pub fn h_get_state(
    env: &mut ContractEnv,
    inst: &mut Instance,
    _caller: &mut CallingFrame,
    input: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    // Ensure we have exactly 1 input parameters
    if input.len() != 1 {
        return Err(CoreError::Execution(
            wasmedge_sdk::error::CoreExecutionError::FuncSigMismatch,
        ));
    }

    // Extract pointer and length for the key
    let item_index = input[0].to_i32() as usize;
    if item_index >= env.state.data.len() {
        return Err(CoreError::Execution(
            wasmedge_sdk::error::CoreExecutionError::ArrayOutOfBounds,
        ));
    }

    let item_data = env.state.data[item_index].clone();
    let item_len = item_data.len() as i32;

    if env.mem_offset + item_len as u32 > MAX_MEMORY_OFFSET {
        return Err(CoreError::Execution(
            wasmedge_sdk::error::CoreExecutionError::MemoryOutOfBounds,
        ));
    }

    let mut mem = inst.get_memory_mut("memory").unwrap();
    let result = mem.set_data(item_data, env.mem_offset);
    if let Err(e) = result {
        println!("Error setting data in memory: {:?}", e);
        return Err(CoreError::Execution(
            wasmedge_sdk::error::CoreExecutionError::FuncSigMismatch,
        ));
    }
    let ptr = env.mem_offset;
    env.mem_offset += item_len as u32;

    Ok(wasm_encoder::value_from_ptr(ptr, item_len))
}

pub fn h_change_state(
    env: &mut ContractEnv,
    inst: &mut Instance,
    _caller: &mut CallingFrame,
    input: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    // Verify input parameters
    if input.len() != 3 {
        return Err(CoreError::Execution(
            wasmedge_sdk::error::CoreExecutionError::FuncSigMismatch,
        ));
    }

    // Extract pointers and lengths for key and value
    let item_index = input[0].to_i32() as usize;
    if item_index >= env.state.data.len() {
        return Err(CoreError::Execution(
            wasmedge_sdk::error::CoreExecutionError::FuncSigMismatch,
        ));
    }

    let value_ptr = input[1].to_i32() as u32;
    let value_len = input[2].to_i32() as u32;

    // Access WASM memory and retrieve the value
    let mem = inst.get_memory_mut("memory").unwrap();

    let value = mem.get_data(value_ptr, value_len).unwrap();
    env.state.data[item_index] = value; // Update the value in storage

    Ok(wasm_encoder::empty_value()) // Return the length as a WasmValue
}

pub fn h_commit_state(
    env: &mut ContractEnv,
    _inst: &mut Instance,
    _caller: &mut CallingFrame,
    _input: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    if env.state.committed {
        return Err(CoreError::Execution(
            wasmedge_sdk::error::CoreExecutionError::FuncSigMismatch,
        ));
    }

    if let Err(e) = save_state(&env.contract_id, &env.state.data) {
        println!("Error saving state: {:?}", e);
        return Err(CoreError::Execution(
            wasmedge_sdk::error::CoreExecutionError::FuncSigMismatch,
        ));
    }

    env.state.committed = true;

    Ok(wasm_encoder::empty_value()) // Return an empty result
}
