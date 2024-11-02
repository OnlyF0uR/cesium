#![no_main]

extern "C" {
    fn h_define_state(storage_len: i32);
    fn h_get_state(item_index: i32) -> i64;
    fn h_change_state(item_index: i32, value_ptr: *const u8, value_len: i32);
    fn h_debug_log(value_ptr: *const u8, value_len: i32);
}

pub fn extract_pointer_length(combined: i64) -> (*const u8, usize) {
    let length = (combined >> 32) as usize; // Extract length
    let ptr = (combined & 0xFFFF_FFFF) as *const u8; // Extract pointer
    (ptr, length)
}

fn compare_bytes(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false; // Different lengths mean they can't be equal
    }

    // Iterate through each byte and compare
    for i in 0..a.len() {
        if a[i] != b[i] {
            return false; // Found a difference
        }
    }
    true // All bytes matched
}

#[no_mangle]
pub unsafe extern "C" fn initialize() -> i32 {
    h_define_state(1);

    // We shall set some state for the first index
    let value = "value";
    let value_ptr = value.as_bytes().as_ptr();
    let value_len = value.len() as i32;
    h_change_state(0, value_ptr, value_len);

    let data_ptr = h_get_state(0);
    let (data_ptr, data_len) = extract_pointer_length(data_ptr);
    if data_len == 0 {
        return 1;
    }
    let buffer: &[u8] = std::slice::from_raw_parts(data_ptr, data_len);
    if !compare_bytes(buffer, value.as_bytes()) {
        return 2;
    }

    unsafe {
        let s = core::str::from_utf8_unchecked(buffer);
        h_debug_log(s.as_ptr(), s.len() as i32);
    }

    // Lets now change the state
    let value = "new_value";
    let value_ptr = value.as_bytes().as_ptr();
    let value_len = value.len() as i32;
    h_change_state(0, value_ptr, value_len);

    let data_ptr = h_get_state(0);
    let (data_ptr, data_len) = extract_pointer_length(data_ptr);
    if data_len == 0 {
        return 1;
    }
    let buffer: &[u8] = std::slice::from_raw_parts(data_ptr, data_len);
    if !compare_bytes(buffer, value.as_bytes()) {
        return 2;
    }

    unsafe {
        let s = core::str::from_utf8_unchecked(buffer);
        h_debug_log(s.as_ptr(), s.len() as i32);
    }

    0
}
