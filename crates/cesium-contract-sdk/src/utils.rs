pub fn unfold_ptr(combined: i64) -> (*const u8, usize) {
    let length = (combined >> 32) as usize; // Extract length
    let ptr = (combined & 0xFFFF_FFFF) as *const u8; // Extract pointer
    (ptr, length)
}
