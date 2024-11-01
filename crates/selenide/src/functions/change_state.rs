use crate::env::ContractEnv;
use wasmer::FunctionEnvMut;

// TODO: Change name to h_write_state
pub fn h_change_state(
    mut env: FunctionEnvMut<ContractEnv>,
    item_index: i32,
    value_ptr: i32,
    value_len: i32,
) {
    let (env, store) = env.data_and_store_mut();

    let mut state_ref = env.state.lock().unwrap();
    if item_index < 0 || item_index as usize >= state_ref.values.len() {
        println!("Invalid state item index");
        return;
    }

    // lets read from memory
    let mem = match &env.memory {
        Some(memory) => memory,
        None => {
            println!("Error getting memory");
            return;
        }
    };

    let mem_view = mem.view(&store);
    let range: std::ops::Range<u64> = value_ptr as u64..(value_ptr + value_len) as u64;
    let value = match mem_view.copy_range_to_vec(range) {
        Ok(value) => value,
        Err(e) => {
            println!("Error reading memory: {:?}", e);
            return;
        }
    };

    state_ref.values[item_index as usize] = value;
}
