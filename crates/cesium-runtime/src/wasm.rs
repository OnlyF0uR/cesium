pub mod wasm_memory {
    use wasmedge_sdk::AsInstance;
    use wasmedge_sdk::{error::CoreError, Instance};
    use wasmedge_sys::{instance::InnerRef, Memory};

    use crate::env::ContractEnv;

    const MAX_MEMORY_OFFSET: u32 = 1 * 64 * 1024; // 65,536 bytes (1 page)

    pub fn allocate(
        inst: &mut Instance,
        env: &mut ContractEnv,
        item_data: &[u8],
    ) -> Result<(u32, i32), CoreError> {
        let item_len = item_data.len() as u32;

        let mut mem = inst.get_memory_mut("memory").unwrap();
        let ptr = find_next_empty_slot(&mem, env.mem_offset, item_len)?;

        let result = mem.set_data(item_data, ptr);
        if let Err(e) = result {
            println!("Error setting data in memory: {:?}", e);
            return Err(CoreError::Execution(
                wasmedge_sdk::error::CoreExecutionError::FuncSigMismatch,
            ));
        }

        env.mem_offset = ptr + item_len;
        Ok((ptr, item_len as i32))
    }

    pub fn allocate_multiple(
        inst: &mut Instance,
        env: &mut ContractEnv,
        items_data: &[Vec<u8>],
    ) -> Result<Vec<(u32, i32)>, CoreError> {
        let mut mem = inst.get_memory_mut("memory").unwrap();

        let mut results = Vec::new();
        for item_data in items_data {
            let item_len = item_data.len() as u32;
            let ptr = find_next_empty_slot(&mem, env.mem_offset, item_len)?;

            let result = mem.set_data(item_data, ptr);
            if let Err(e) = result {
                println!("Error setting data in memory: {:?}", e);
                return Err(CoreError::Execution(
                    wasmedge_sdk::error::CoreExecutionError::FuncSigMismatch,
                ));
            }

            env.mem_offset = ptr + item_len;
            results.push((ptr, item_len as i32));
        }

        Ok(results)
    }

    fn find_next_empty_slot(
        mem: &InnerRef<Memory, &mut Instance>,
        start_offset: u32,
        length: u32,
    ) -> Result<u32, CoreError> {
        // Preliminary check for obvious out of bounds
        const MAX_SEARCH_LIMIT: u32 = MAX_MEMORY_OFFSET;
        let max_check_length = MAX_SEARCH_LIMIT.saturating_sub(start_offset);
        if length > max_check_length {
            return Err(CoreError::Execution(
                wasmedge_sdk::error::CoreExecutionError::MemoryOutOfBounds,
            ));
        }

        let mut current_offset = start_offset;
        while current_offset + length <= MAX_MEMORY_OFFSET {
            let mut is_region_empty = true;
            let mut continuous_length = 0;

            // Check the entire region of 'length' size
            while continuous_length < length && is_region_empty {
                let chunk_size = std::cmp::min(
                    length - continuous_length,
                    MAX_MEMORY_OFFSET - (current_offset + continuous_length),
                );

                if chunk_size == 0 {
                    return Err(CoreError::Execution(
                        wasmedge_sdk::error::CoreExecutionError::MemoryOutOfBounds,
                    ));
                }

                let data = mem
                    .get_data(current_offset + continuous_length, chunk_size)
                    .map_err(|_| {
                        CoreError::Execution(
                            wasmedge_sdk::error::CoreExecutionError::MemoryOutOfBounds,
                        )
                    })?;

                if data.iter().any(|&byte| byte != 0) {
                    is_region_empty = false;
                    // Skip to the first non-zero byte position
                    if let Some(first_non_zero) = data.iter().position(|&byte| byte != 0) {
                        current_offset += continuous_length + first_non_zero as u32 + 1;
                    } else {
                        current_offset += chunk_size;
                    }
                    break;
                }

                continuous_length += chunk_size;
            }

            if is_region_empty && continuous_length == length {
                return Ok(current_offset);
            }

            // If we didn't find a complete empty region, continue from the next position
            if continuous_length == 0 {
                current_offset += 1;
            }
            // else current_offset was already updated in the inner loop
        }

        Err(CoreError::Execution(
            wasmedge_sdk::error::CoreExecutionError::MemoryOutOfBounds,
        ))
    }
}

pub mod wasm_values {
    use wasmedge_sdk::WasmValue;

    pub fn from_ptr(ptr: u32, len: i32) -> Vec<WasmValue> {
        vec![WasmValue::from_i64(((len as i64) << 32) | (ptr as i64))]
    }

    pub fn from_ptrs(ptr1: u32, len1: u32, ptr2: u32, len2: u32) -> Vec<WasmValue> {
        let combined: i128 = ((len2 as i128) << 96)
            | ((ptr2 as i128) << 64)
            | ((len1 as i128) << 32)
            | (ptr1 as i128);

        vec![WasmValue::from_v128(combined)]
    }

    pub fn empty() -> Vec<WasmValue> {
        vec![]
    }
}
