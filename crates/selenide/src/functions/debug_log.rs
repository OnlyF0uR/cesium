use crate::env::ContractEnv;
use wasmer::FunctionEnvMut;

pub fn h_debug_log(mut env: FunctionEnvMut<ContractEnv>, value_ptr: i32, value_len: i32) {
    let (env, store) = env.data_and_store_mut();

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

    println!("Debug log ({}): {:?}", value.len(), value);
    // Print as string if possible
    match std::str::from_utf8(&value) {
        Ok(string) => {
            println!(" - String value: {}", string);
        }
        Err(_) => {
            return;
        }
    };
}
