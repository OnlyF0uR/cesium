use cesium_crypto::id::generate_id;
use wasmer::FunctionEnvMut;

use crate::{env::ContractEnv, memory};

pub fn h_gen_id(mut env: FunctionEnvMut<ContractEnv>) -> i64 {
    let (contract_env, store) = env.data_and_store_mut();

    let token_id = generate_id();
    let (ptr, len) = match memory::allocate(contract_env, &store, &token_id) {
        Ok((ptr, len)) => (ptr, len),
        Err(e) => {
            println!("Error allocating memory: {:?}", e);
            return 0;
        }
    };

    memory::value_from_ptr(ptr, len)
}
