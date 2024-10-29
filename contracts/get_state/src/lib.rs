// external c function for get_state
extern "C" {
    // Pasa pointer to the key value and the length of the key
    // This is so we know how much to allocate
    fn h_get_state(key_ptr: *const u8, key_len: i32) -> i32;
    // Write the state to memory
    // This is for writing to the buffer
    fn h_write_state_mem(ptr: *const u8);
}

#[no_mangle]
pub unsafe extern "C" fn entry_proc() {
    let key = "example_key";
    let key_ptr = key.as_bytes().as_ptr();
    let key_len = key.len() as i32;

    let data_len = h_get_state(key_ptr, key_len) as usize;
    let mut buffer = Vec::with_capacity(data_len);
    let buffer_ptr = buffer.as_mut_ptr();
    // Write to the buffer
    h_write_state_mem(buffer_ptr);

    let s = std::str::from_utf8(&buffer).unwrap();
    println!("Retrieved state value len: {}", s.len());
}
