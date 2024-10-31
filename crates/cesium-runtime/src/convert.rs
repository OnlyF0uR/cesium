pub mod wasm_encoder {
    use wasmedge_sdk::WasmValue;

    pub fn value_from_ptr(ptr: u32, len: i32) -> Vec<WasmValue> {
        vec![WasmValue::from_i64(((len as i64) << 32) | (ptr as i64))]
    }

    pub fn value_from_ptrs(ptr1: u32, len1: u32, ptr2: u32, len2: u32) -> Vec<WasmValue> {
        let combined: i128 = ((len2 as i128) << 96)
            | ((ptr2 as i128) << 64)
            | ((len1 as i128) << 32)
            | (ptr1 as i128);

        vec![WasmValue::from_v128(combined)]
    }

    pub fn empty_value() -> Vec<WasmValue> {
        vec![]
    }
}
