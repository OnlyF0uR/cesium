use cesium_material::id::generate_id;
use wasmedge_sdk::{error::CoreError, CallingFrame, Instance, WasmValue};
use wasmedge_sys::AsInstance;

use crate::{
    data::{save_account_data, save_state, MAX_MEMORY_OFFSET},
    env::ContractEnv,
    wasm::wasm_values,
};

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

    Ok(wasm_values::empty())
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

    Ok(wasm_values::from_ptr(ptr, token_id_len))
}
