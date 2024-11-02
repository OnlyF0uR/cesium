use std::slice;

use selenide_sdk::data::State;
use selenide_sdk_macros::cesium;

#[cesium(contract_state)]
pub struct StateManager {
    example_str: String,
}

#[cesium(external_func)]
pub fn initialize() -> i32 {
    StateManager::define_all();

    let example_value = "example_value";

    let mut mng = StateManager::new();
    if let Err(_) = mng.set_example_str(example_value) {
        return 1;
    }

    match mng.get_example_str() {
        Ok(value) => {
            if &value != example_value {
                println!("State value is {} (expected: my_value)", value);
                return 1;
            }
        }
        Err(_) => return 1,
    }

    0
}

#[cesium(external_func)]
pub fn compare_state(value_ptr: *const u8, value_len: i32) -> i32 {
    let value = unsafe { slice::from_raw_parts(value_ptr, value_len as usize) };
    let value = std::str::from_utf8(value).unwrap();

    let mut mng = StateManager::new();
    match mng.get_example_str() {
        Ok(v) => {
            if v != value {
                println!("State value is {} (expected: {})", v, value);
                return 1;
            }
        }
        Err(_) => return 1,
    }

    0
}
