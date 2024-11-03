use wasmer::FunctionEnvMut;

use crate::env::ContractEnv;

pub fn h_get_account_data(
    _env: FunctionEnvMut<ContractEnv>,
    _address_ptr: i32,
    _address_len: i32,
) -> i64 {
    // TODO: This
    todo!();
}
