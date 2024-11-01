use wasmer::{Memory, StoreMut};

use crate::{env::ContractEnv, errors::RuntimeError};

const MAX_MEMORY_OFFSET: u64 = 1 * 64 * 1024; // 65,536 bytes (1 page)

pub fn allocate(
    env: &mut ContractEnv,
    store: &StoreMut,
    item_data: &[u8],
) -> Result<(u64, u64), RuntimeError> {
    let item_len = item_data.len() as u64;

    let mem = match env.memory {
        Some(ref mut mem) => mem,
        None => return Err(RuntimeError::MemoryNotInitialized),
    };

    let mut offset_ref = env.mem_offset.lock().unwrap();
    let ptr = find_next_empty_slot(&mem, store, *offset_ref, item_len)?;

    let view = mem.view(store);
    view.write(ptr, item_data)?;

    *offset_ref += ptr + item_len;
    println!("Allocated memory at offset: {}::{}", ptr, item_len);
    Ok((ptr, item_len))
}

fn find_next_empty_slot(
    mem: &Memory,
    store: &StoreMut,
    start_offset: u64,
    length: u64,
) -> Result<u64, RuntimeError> {
    // Preliminary check for obvious out of bounds
    const MAX_SEARCH_LIMIT: u64 = MAX_MEMORY_OFFSET;
    let max_check_length = MAX_SEARCH_LIMIT.saturating_sub(start_offset);
    if length > max_check_length {
        return Err(RuntimeError::MemoryOutOfBounds);
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
                return Err(RuntimeError::MemoryOutOfBounds);
            }

            let range: std::ops::Range<u64> = (current_offset + continuous_length)
                ..(current_offset + continuous_length + chunk_size);
            let data = mem.view(store).copy_range_to_vec(range)?;
            if data.iter().any(|&byte| byte != 0) {
                is_region_empty = false;
                // Skip to the first non-zero byte position
                if let Some(first_non_zero) = data.iter().position(|&byte| byte != 0) {
                    current_offset += continuous_length + first_non_zero as u64 + 1;
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

    Err(RuntimeError::MemoryOutOfBounds)
}

pub fn value_from_ptr(ptr: u64, len: u64) -> i64 {
    ((len as i64) << 32) | (ptr as i64)
}

pub fn value_from_ptrs(ptr1: u64, len1: u64, ptr2: u64, len2: u64) -> i128 {
    ((len2 as i128) << 96) | ((ptr2 as i128) << 64) | ((len1 as i128) << 32) | (ptr1 as i128)
}
