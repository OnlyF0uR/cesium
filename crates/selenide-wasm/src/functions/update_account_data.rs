use wasmer::FunctionEnvMut;

use crate::env::ContractEnv;

pub fn h_update_account_data(
    _env: FunctionEnvMut<ContractEnv>,
    _address_ptr: i32,
    _address_len: i32,
    _data_ptr: i32,
    _data_len: i32,
) {
    // TODO: This
    todo!();
}
