use wasmer::FunctionEnvMut;

use crate::env::ContractEnv;

pub fn h_define_state(env: FunctionEnvMut<ContractEnv>, state_count: i32) {
    let mut state_ref = env.data().state.lock().unwrap();
    if state_ref.values.len() > 0 {
        println!("State already defined");
        return;
    }

    // Create a new vector with all the required state items
    state_ref.values = vec![Vec::new(); state_count as usize];
}
