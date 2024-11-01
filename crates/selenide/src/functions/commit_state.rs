use wasmer::FunctionEnvMut;

use crate::env::ContractEnv;

pub fn h_commit_state(env: FunctionEnvMut<ContractEnv>) {
    let state_ref = env.data().state.lock().unwrap();
    println!("Committing state: {:?}", state_ref.values);
}
