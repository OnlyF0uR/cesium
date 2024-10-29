// external c function for get_state
extern "C" {
    // Pasa pointer to the key value and the length of the key
    // This is so we know how much to allocate
    fn h_get_state(key_ptr: *const u8, key_len: i32) -> i32;
    // Write the state to memory
    // This is for writing to the buffer
    fn h_write_state_mem(ptr: *mut u8);

    fn h_change_state(key_ptr: *const u8, key_len: i32, value_ptr: *const u8, value_len: i32);
}

#[no_mangle]
pub unsafe extern "C" fn entry_proc() -> i32 {
    let key = "example_key";
    let key_ptr = key.as_bytes().as_ptr();
    let key_len = key.len() as i32;

    let data_len = h_get_state(key_ptr, key_len) as usize;
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
    let new_key = "new_key";
    let new_value = "new_value";
    let new_key_ptr = new_key.as_bytes().as_ptr();
    let new_key_len = new_key.len() as i32;
    let new_value_ptr = new_value.as_bytes().as_ptr();
    let new_value_len = new_value.len() as i32;

    h_change_state(new_key_ptr, new_key_len, new_value_ptr, new_value_len);

    // Create a buffer for reading
    let data_len = h_get_state(new_key_ptr, new_key_len) as usize;
    let mut buffer = Vec::with_capacity(data_len);

    // Write to the buffer
    h_write_state_mem(buffer.as_mut_ptr());
    buffer.set_len(data_len);

    let s = std::str::from_utf8(&buffer).unwrap();
    if s != new_value {
        println!("State value is {} (expected: new_value)", s);
        return 1;
    }

    return 0;
}
