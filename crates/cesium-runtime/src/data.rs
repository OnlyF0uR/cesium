use std::collections::HashMap;

use crate::env::AccountDataItem;

pub const MAX_MEMORY_OFFSET: u32 = 1 * 64 * 1024; // 65,536 bytes (1 page)

pub fn load_state(
    _contract: &[u8],
) -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error + Send + Sync>> {
    // TODO: Implement this function
    Ok(Vec::new())
}

pub fn save_state(
    contract: &[u8],
    state: &Vec<Vec<u8>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Saving for contract {:?}, state: {:?}", contract, state);
    // TODO: Implement this function
    Ok(())
}

pub fn load_account_data(
    _contract: &[u8],
    _account: &[u8],
) -> Result<AccountDataItem, Box<dyn std::error::Error + Send + Sync>> {
    // TODO: Implement this function
    Ok(AccountDataItem {
        owner: Vec::new(),
        update_auth: Vec::new(),
        data: Vec::new(),
    })
}

pub fn save_account_data(
    contract: &[u8],
    data: &HashMap<Vec<u8>, AccountDataItem>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Saving for contract {:?}, data: {:?}", contract, data);
    // TODO: Implement this function
    Ok(())
}
