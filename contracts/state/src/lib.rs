extern "C" {
    fn h_define_state(storage_len: i32);
    fn h_get_state(item_index: i32) -> i32;
    fn h_write_state_mem(ptr: *mut u8);
    fn h_change_state(item_index: i32, value_ptr: *const u8, value_len: i32);
    fn h_commit_state();
}

#[no_mangle]
pub unsafe extern "C" fn initialize() -> i32 {
    h_define_state(1);

    let data_len = h_get_state(0) as usize;
    let mut buffer = Vec::with_capacity(data_len);
    // Write to the buffer
    h_write_state_mem(buffer.as_mut_ptr());
    buffer.set_len(data_len);

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
    let data_len = h_get_state(0) as usize;
    let mut buffer = Vec::with_capacity(data_len);

    // Write to the buffer
    h_write_state_mem(buffer.as_mut_ptr());
    buffer.set_len(data_len);

    let s = std::str::from_utf8(&buffer).unwrap();
    if s != new_value {
        println!("State value is {} (expected: new_value)", s);
        return 1;
    }

    h_commit_state();

    0
}
