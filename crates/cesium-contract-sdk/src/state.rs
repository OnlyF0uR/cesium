extern "C" {
    fn h_get_state(key_ptr: *const u8, key_len: i32) -> i32;
    fn h_write_state_mem(ptr: *mut u8);
    fn h_change_state(key_ptr: *const u8, key_len: i32, value_ptr: *const u8, value_len: i32);
    fn h_commit_state();
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
    pub fn get(key: &str) -> Result<Option<Vec<u8>>, StateError> {
        unsafe {
            let key_ptr = key.as_bytes().as_ptr();
            let key_len = key.len() as i32;

            let data_len = h_get_state(key_ptr, key_len) as usize;
            if data_len == 0 {
                return Ok(None);
            }

            let mut buffer: Vec<u8> = Vec::with_capacity(data_len);
            h_write_state_mem(buffer.as_mut_ptr());
            buffer.set_len(data_len);

            Ok(Some(buffer))
        }
    }

    pub fn set(key: &str, value: &[u8]) -> Result<(), StateError> {
        unsafe {
            let key_ptr = key.as_bytes().as_ptr();
            let key_len = key.len() as i32;
            let value_ptr: *const u8 = value.as_ptr();
            let value_len = value.len() as i32;

            h_change_state(key_ptr, key_len, value_ptr, value_len);
        }

        Ok(())
    }

    pub fn commit() {
        unsafe {
            h_commit_state();
        }
    }
}
