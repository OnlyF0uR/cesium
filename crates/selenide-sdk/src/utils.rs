// Due to the limitations of returning parameters from host to wasm
// we have to get creative in passing pointers and the length of the data
// they point to.

pub fn unfold_ptr(combined: i64) -> (*const u8, usize) {
    let length = (combined >> 32) as usize; // Extract length
    let ptr = (combined & 0xFFFF_FFFF) as *const u8; // Extract pointer
    (ptr, length)
}

pub fn unfold_ptrs(encoded: i128) -> (u32, u32, u32, u32) {
    let ptr1 = (encoded & 0xFFFFFFFF) as u32;
    let len1 = ((encoded >> 32) & 0xFFFFFFFF) as u32;
    let ptr2 = ((encoded >> 64) & 0xFFFFFFFF) as u32;
    let len2 = ((encoded >> 96) & 0xFFFFFFFF) as u32;
    (ptr1, len1, ptr2, len2)
}
