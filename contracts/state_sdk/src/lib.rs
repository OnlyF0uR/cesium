use std::slice;

use cesium_contract_sdk::state::State;

#[no_mangle]
pub extern "C" fn initialize() -> i32 {
    let value = "my_value";

    // Store raw string data
    if let Err(_) = State::set("my_key", value.as_bytes()) {
        return 1;
    }

    0
}

#[no_mangle]
pub extern "C" fn compare_state(
    key_ptr: *const u8,
    key_len: i32,
    value_ptr: *const u8,
    value_len: i32,
) -> i32 {
    // print pointers
    // println!("Key pointer: {:p}, Value pointer: {:p}", key_ptr, value_ptr);

    // get vec<u8> from pointer
    let key = unsafe { slice::from_raw_parts(key_ptr, key_len as usize) };
    let key = std::str::from_utf8(key).unwrap();

    let value = unsafe { slice::from_raw_parts(value_ptr, value_len as usize) };
    let value = std::str::from_utf8(value).unwrap();

    println!("Key: {}, Value: {}", key, value);

    match State::get(key) {
        Ok(Some(v)) => {
            let s = std::str::from_utf8(&v).unwrap();
            if s != value {
                println!("State value is {} (expected: my_value)", s);
                return 1;
            }
        }
        Ok(None) => println!("Key not found"),
        Err(_) => return 1,
    }

    0
}
