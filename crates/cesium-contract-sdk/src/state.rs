extern "C" {
    fn h_get_state(key_ptr: *const u8, key_len: i32) -> i32;
    fn h_write_state_mem(ptr: *mut u8);
    fn h_change_state(key_ptr: *const u8, key_len: i32, value_ptr: *const u8, value_len: i32);
}

#[derive(Debug)]
pub enum StateError {
    SerializationError(String),
    DeserializationError(String),
    InvalidUtf8,
}

pub struct State;

impl State {
    pub fn new() -> Self {
        Self
    }

    pub fn get(&self, key: &str) -> Result<Option<Vec<u8>>, StateError> {
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

    pub fn set(&self, key: &str, value: &[u8]) -> Result<(), StateError> {
        unsafe {
            let key_ptr = key.as_bytes().as_ptr();
            let key_len = key.len() as i32;
            let value_ptr = value.as_ptr();
            let value_len = value.len() as i32;

            h_change_state(key_ptr, key_len, value_ptr, value_len);
        }

        Ok(())
    }
}

pub trait StateAccess {
    fn state(&self) -> &State;

    // Similar to, but then bin instead of json
    // fn get_json<T: serde::de::DeserializeOwned>(&self, key: &str) -> Result<Option<T>, StateError> {
    //     match self.state().get(key)? {
    //         Some(value) => serde_json::from_str(&value)
    //             .map(Some)
    //             .map_err(|e| StateError::DeserializationError(e.to_string())),
    //         None => Ok(None),
    //     }
    // }

    // fn set_json<T: serde::Serialize>(&self, key: &str, value: &T) -> Result<(), StateError> {
    //     let serialized = serde_json::to_string(value)
    //         .map_err(|e| StateError::SerializationError(e.to_string()))?;
    //     self.state().set(key, &serialized)
    // }
    // fn get_bin() {}
    // fn set_bin() {}
}
