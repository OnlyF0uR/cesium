use wasmedge_sdk::{error::CoreError, CallingFrame, Instance, WasmValue};
use wasmedge_sys::AsInstance;

use crate::{convert::wasm_encoder, data::save_account_data, env::ContractEnv};

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
