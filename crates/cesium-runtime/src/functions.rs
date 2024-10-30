use wasmedge_sdk::{error::CoreError, CallingFrame, Instance, WasmValue};
use wasmedge_sys::AsInstance;

use crate::state::ContractState;

pub fn h_define_state(
    state: &mut ContractState,
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

    if state.storage_initialized {
        return Err(CoreError::Execution(
            wasmedge_sdk::error::CoreExecutionError::FuncSigMismatch,
        ));
    }

    state.storage_initialized = true;

    let storage_len = input[0].to_i32() as usize;
    state.storage = vec![Vec::new(); storage_len];

    Ok(vec![])
}

// Host function to get a value from storage by key
pub fn h_get_state(
    state: &mut ContractState,
    _inst: &mut Instance,
    _caller: &mut CallingFrame,
    input: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    // Ensure we have exactly 2 input parameters
    if input.len() != 1 {
        return Err(CoreError::Execution(
            wasmedge_sdk::error::CoreExecutionError::FuncSigMismatch,
        ));
    }

    // Extract pointer and length for the key
    let item_index = input[0].to_i32() as usize;
    if item_index >= state.storage.len() {
        return Err(CoreError::Execution(
            wasmedge_sdk::error::CoreExecutionError::FuncSigMismatch,
        ));
    }

    let item_data = state.storage[item_index].clone();
    let item_len = item_data.len() as i32;

    state.get_state_result = item_data;
    Ok(vec![WasmValue::from_i32(item_len)])
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
    state.get_state_result.clear(); // Clear the cached result

    // read from memory given the offset
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
    if input.len() != 3 {
        return Err(CoreError::Execution(
            wasmedge_sdk::error::CoreExecutionError::FuncSigMismatch,
        ));
    }

    // Extract pointers and lengths for key and value
    let item_index = input[0].to_i32() as usize;
    if item_index >= state.storage.len() {
        return Err(CoreError::Execution(
            wasmedge_sdk::error::CoreExecutionError::FuncSigMismatch,
        ));
    }

    let value_ptr = input[1].to_i32() as u32;
    let value_len = input[2].to_i32() as u32;

    // Access WASM memory and retrieve the value
    let mem = inst.get_memory_mut("memory").unwrap();

    let value = mem.get_data(value_ptr, value_len).unwrap();
    state.storage[item_index] = value; // Update the value in storage

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

pub fn h_initialize_data_account(
    state: &mut ContractState,
    inst: &mut Instance,
    _caller: &mut CallingFrame,
    input: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    // The input:
    // - owner address (ptr, len)
    // - initial data (ptr to vec<u8>, len)
    if input.len() != 4 {
        return Err(CoreError::Execution(
            wasmedge_sdk::error::CoreExecutionError::FuncSigMismatch,
        ));
    }

    let owner_ptr = input[0].to_i32() as u32;
    let owner_len = input[1].to_i32() as u32;

    let data_ptr = input[2].to_i32() as u32;
    let data_len = input[3].to_i32() as u32;

    let mem = inst.get_memory_mut("memory").unwrap();
    let owner = mem.get_data(owner_ptr, owner_len).unwrap();
    let update_auth = state.contract_id.clone();
    let data = mem.get_data(data_ptr, data_len).unwrap();

    println!("Owner: {:?}", owner);
    println!("Update Auth: {:?}", update_auth);
    println!("Data: {:?}", data);

    // TODO: This
    // dont forget to set the get_address_result for the new created account, so we can
    // get it

    // Will return the address the len of the address of
    // a new account, so wasm can call h_write_address_mem
    Ok(vec![WasmValue::from_i32(0)])
}

pub fn h_initialize_independent_data_account(
    _state: &mut ContractState,
    inst: &mut Instance,
    _caller: &mut CallingFrame,
    input: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    // The input:
    // - owner address (ptr, len)
    // - update authority address (ptr, len)
    // - initial data (ptr to vec<u8>, len)
    if input.len() != 6 {
        return Err(CoreError::Execution(
            wasmedge_sdk::error::CoreExecutionError::FuncSigMismatch,
        ));
    }

    let owner_ptr = input[0].to_i32() as u32;
    let owner_len = input[1].to_i32() as u32;

    let update_auth_ptr = input[2].to_i32() as u32;
    let update_auth_len = input[3].to_i32() as u32;

    let data_ptr = input[4].to_i32() as u32;
    let data_len = input[5].to_i32() as u32;

    let mem = inst.get_memory_mut("memory").unwrap();
    let owner = mem.get_data(owner_ptr, owner_len).unwrap();
    let update_auth = mem.get_data(update_auth_ptr, update_auth_len).unwrap();
    let data = mem.get_data(data_ptr, data_len).unwrap();

    println!("Owner: {:?}", owner);
    println!("Update Auth: {:?}", update_auth);
    println!("Data: {:?}", data);

    // TODO: This
    // dont forget to set the get_address_result for the new created account, so we can
    // get it

    // Will return the address the len of the address of
    // a new account, so wasm can call h_write_address_mem
    Ok(vec![WasmValue::from_i32(0)])
}

pub fn h_write_address_mem(
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

    // Get the cached address of the new account
    let value = state.get_address_result.clone();

    // Write the cached address to memory at the given offset
    mem.set_data(value.clone(), offset as u32).unwrap();
    state.get_address_result.clear(); // Clear the cached result

    Ok(vec![])
}

pub fn h_update_data_account(
    state: &mut ContractState,
    inst: &mut Instance,
    _caller: &mut CallingFrame,
    input: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    // The input:
    // - account address (ptr, len)
    // - new data (ptr to vec<u8>, len)

    if input.len() != 2 {
        return Err(CoreError::Execution(
            wasmedge_sdk::error::CoreExecutionError::FuncSigMismatch,
        ));
    }

    let account_ptr = input[0].to_i32() as u32;
    let account_len = input[1].to_i32() as u32;

    let data_ptr = input[2].to_i32() as u32;
    let data_len = input[3].to_i32() as u32;

    let mem = inst.get_memory_mut("memory").unwrap();
    let account = mem.get_data(account_ptr, account_len).unwrap();
    let account_str = std::str::from_utf8(&account).unwrap();
    if account_str == &state.contract_id || account_str == &state.caller_id {
        // We already now that this is impossible
        return Err(CoreError::Execution(
            wasmedge_sdk::error::CoreExecutionError::FuncSigMismatch,
        ));
    }

    let data = mem.get_data(data_ptr, data_len).unwrap();
    println!("New data: {:?}", data);

    // TODO: Set the account state if it exists

    Ok(vec![])
}
