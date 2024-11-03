use crate::utils::unfold_ptr;

extern "C" {
    fn h_define_state(storage_len: i32);
    fn h_get_state(item_index: i32) -> i64;
    fn h_write_state(item_index: i32, value_ptr: *const u8, value_len: i32);
    fn h_initialize_data_account(
        owner_ptr: *const u8,
        owner_len: i32,
        data_ptr: *const u8,
        data_len: i32,
    ) -> i64;
    fn h_initialize_independent_data_account(
        owner_ptr: *const u8,
        owner_len: i32,
        data_ptr: *const u8,
        data_len: i32,
        contract_id_ptr: *const u8,
        contract_id_len: i32,
    ) -> i64;
    fn h_update_data_account(
        owner_ptr: *const u8,
        owner_len: i32,
        data_ptr: *const u8,
        data_len: i32,
    );
}

#[derive(Debug)]
pub enum StateError {
    SerializationError(String),
    DeserializationError(String),
    InvalidUtf8,
    NoReturnData,
}

pub struct State;

impl State {
    pub fn define(storage_len: i32) {
        unsafe {
            h_define_state(storage_len);
        }
    }

    pub fn get(item_index: i32) -> Result<Option<&'static [u8]>, StateError> {
        unsafe {
            let result = h_get_state(item_index);
            let (data_ptr, data_len) = unfold_ptr(result);
            if data_len == 0 {
                return Ok(None);
            }

            // Read the value from memory
            let value = std::slice::from_raw_parts(data_ptr, data_len);
            Ok(Some(value))
        }
    }

    pub fn set(item_index: i32, value: &[u8]) -> Result<(), StateError> {
        unsafe {
            let value_ptr: *const u8 = value.as_ptr();
            let value_len = value.len() as i32;

            h_write_state(item_index, value_ptr, value_len);
        }

        Ok(())
    }
}

pub struct DataAccount;
impl DataAccount {
    pub fn create(owner: &[u8], data: &[u8]) -> Result<Option<&'static [u8]>, StateError> {
        unsafe {
            let owner_ptr: *const u8 = owner.as_ptr();
            let owner_len = owner.len() as i32;
            let data_ptr: *const u8 = data.as_ptr();
            let data_len = data.len() as i32;

            let result = h_initialize_data_account(owner_ptr, owner_len, data_ptr, data_len);
            let (addr_ptr, addr_len) = unfold_ptr(result);
            if addr_len == 0 {
                return Ok(None);
            }

            let value = std::slice::from_raw_parts(addr_ptr as *const u8, data_len as usize);
            Ok(Some(value))
        }
    }

    pub fn create_independent(
        owner: &[u8],
        data: &[u8],
        contract_id: &[u8],
    ) -> Result<Option<&'static [u8]>, StateError> {
        unsafe {
            let owner_ptr: *const u8 = owner.as_ptr();
            let owner_len = owner.len() as i32;
            let data_ptr: *const u8 = data.as_ptr();
            let data_len = data.len() as i32;
            let contract_id_ptr: *const u8 = contract_id.as_ptr();
            let contract_id_len = contract_id.len() as i32;

            let result = h_initialize_independent_data_account(
                owner_ptr,
                owner_len,
                data_ptr,
                data_len,
                contract_id_ptr,
                contract_id_len,
            );
            let (addr_ptr, addr_len) = unfold_ptr(result);

            if addr_len == 0 {
                return Ok(None);
            }

            let value = std::slice::from_raw_parts(addr_ptr as *const u8, data_len as usize);
            Ok(Some(value))
        }
    }

    pub fn update(owner: &[u8], data: &[u8]) -> Result<(), StateError> {
        unsafe {
            let owner_ptr: *const u8 = owner.as_ptr();
            let owner_len = owner.len() as i32;
            let data_ptr: *const u8 = data.as_ptr();
            let data_len = data.len() as i32;

            h_update_data_account(owner_ptr, owner_len, data_ptr, data_len);
        }

        Ok(())
    }
}
