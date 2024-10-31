use cesium_material::id::generate_id;
use wasmedge_sdk::{error::CoreError, CallingFrame, Instance, WasmValue};
use wasmedge_sys::AsInstance;

use crate::{
    convert::wasm_encoder,
    data::{save_account_data, save_state, MAX_MEMORY_OFFSET},
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

pub fn h_initialize_data_account(
    env: &mut ContractEnv,
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
    let update_auth = env.caller_id.clone();
    let data = mem.get_data(data_ptr, data_len).unwrap();

    println!("Owner: {:?}", owner);
    println!("Update Auth: {:?}", update_auth);
    println!("Data: {:?}", data);

    // TODO: This
    // dont forget to set the get_address_result for the new created account, so we can
    // get it

    // TODO: Set in memory and return ptr + len

    // Will return the address the len of the address of
    // a new account, so wasm can call h_write_address_mem
    Ok(wasm_encoder::value_from_ptr(0, 0))
}

pub fn h_initialize_independent_data_account(
    _env: &mut ContractEnv,
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

    // TODO: Set in memory and return ptr + len

    // Will return the address the len of the address of
    // a new account, so wasm can call h_write_address_mem
    Ok(wasm_encoder::value_from_ptr(0, 0))
}

pub fn h_update_data_account(
    env: &mut ContractEnv,
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
    if account == env.contract_id || account == env.caller_id {
        // We already now that this is impossible
        return Err(CoreError::Execution(
            wasmedge_sdk::error::CoreExecutionError::FuncSigMismatch,
        ));
    }

    let data = mem.get_data(data_ptr, data_len).unwrap();
    println!("New data: {:?}", data);

    // TODO: Set the account state if it exists

    Ok(wasm_encoder::empty_value())
}

pub fn h_commit_account_data(
    env: &mut ContractEnv,
    _inst: &mut Instance,
    _caller: &mut CallingFrame,
    _input: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    if env.account_data.committed {
        return Err(CoreError::Execution(
            wasmedge_sdk::error::CoreExecutionError::FuncSigMismatch,
        ));
    }

    if let Err(e) = save_account_data(&env.contract_id, &env.account_data.data) {
        println!("Error saving account data: {:?}", e);
        return Err(CoreError::Execution(
            wasmedge_sdk::error::CoreExecutionError::FuncSigMismatch,
        ));
    }

    env.account_data.committed = true;

    Ok(vec![]) // Return an empty result
}

pub fn h_commit_all(
    env: &mut ContractEnv,
    _inst: &mut Instance,
    _caller: &mut CallingFrame,
    _input: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    if env.state.committed || env.account_data.committed {
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

    if let Err(e) = save_account_data(&env.contract_id, &env.account_data.data) {
        println!("Error saving account data: {:?}", e);
        return Err(CoreError::Execution(
            wasmedge_sdk::error::CoreExecutionError::FuncSigMismatch,
        ));
    }

    env.account_data.committed = true;

    Ok(wasm_encoder::empty_value())
}

pub fn h_gen_id(
    env: &mut ContractEnv,
    inst: &mut Instance,
    _caller: &mut CallingFrame,
    _input: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    let token_id = generate_id();
    let token_id_len = token_id.len() as i32;

    if env.mem_offset + token_id_len as u32 > MAX_MEMORY_OFFSET {
        return Err(CoreError::Execution(
            wasmedge_sdk::error::CoreExecutionError::MemoryOutOfBounds,
        ));
    }

    let mut mem = inst.get_memory_mut("memory").unwrap();
    let result = mem.set_data(token_id, env.mem_offset);
    if let Err(e) = result {
        println!("Error setting data in memory: {:?}", e);
        return Err(CoreError::Execution(
            wasmedge_sdk::error::CoreExecutionError::FuncSigMismatch,
        ));
    }
    let ptr = env.mem_offset;
    env.mem_offset += token_id_len as u32;

    Ok(wasm_encoder::value_from_ptr(ptr, token_id_len))
}
