extern "C" {
    fn h_define_state(storage_len: i32);
    fn h_get_state(item_index: i32) -> i64;
    fn h_change_state(item_index: i32, value_ptr: *const u8, value_len: i32);
    fn h_commit_state();
}

pub fn extract_pointer_length(combined: i64) -> (*const u8, usize) {
    let length = (combined >> 32) as usize; // Extract length
    let ptr = (combined & 0xFFFF_FFFF) as *const u8; // Extract pointer
    (ptr, length)
}

#[no_mangle]
pub unsafe extern "C" fn initialize() -> i32 {
    h_define_state(1);

    let data_ptr = h_get_state(0);
    let (data_ptr, data_len) = extract_pointer_length(data_ptr);
    let buffer = std::slice::from_raw_parts(data_ptr, data_len);

    let s = std::str::from_utf8(&buffer).unwrap();
    if s.len() != 0 {
        println!("State value is {} (expected: empty)", s);
        return 1;
    }

    // Lets now change the state
    let new_value = "new_value";
    let new_value_ptr = new_value.as_bytes().as_ptr();
    let new_value_len = new_value.len() as i32;

    h_change_state(0, new_value_ptr, new_value_len);

    // Create a buffer for reading
    let data_ptr = h_get_state(0);
    let (data_ptr, data_len) = extract_pointer_length(data_ptr);
    let buffer = std::slice::from_raw_parts(data_ptr, data_len);

    let s = std::str::from_utf8(&buffer).unwrap();
    if s != new_value {
        println!("State value is {} (expected: new_value)", s);
        return 1;
    }

    h_commit_state();

    0
}
