pub mod wasm_encoder {
    use wasmedge_sdk::WasmValue;

    pub fn value_from_ptr(ptr: u32, len: i32) -> Vec<WasmValue> {
        vec![WasmValue::from_i64(((len as i64) << 32) | (ptr as i64))]
    }

    pub fn empty_value() -> Vec<WasmValue> {
        vec![]
    }
}
