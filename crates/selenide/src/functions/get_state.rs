use wasmer::FunctionEnvMut;

use crate::{env::ContractEnv, memory};

pub fn h_get_state(mut env: FunctionEnvMut<ContractEnv>, state_item_index: i32) -> i64 {
    let (contract_env, store) = env.data_and_store_mut();

    let state_ref = match contract_env.state.lock() {
        Ok(state_ref) => state_ref,
        Err(poisoned) => {
            println!("Error locking state: {:?}", poisoned);
            return 0;
        }
    };
    if state_item_index < 0 || state_item_index >= state_ref.clone().values.len() as i32 {
        println!("Invalid state item index");
        return 0;
    }

    let item_data = state_ref.values[state_item_index as usize].clone();
    drop(state_ref);
    let (ptr, len) = match memory::allocate(contract_env, &store, &item_data) {
        Ok((ptr, len)) => (ptr, len),
        Err(e) => {
            println!("Error allocating memory: {:?}", e);
            return 0;
        }
    };

    memory::value_from_ptr(ptr, len)
}
