use std::collections::HashMap;

// State types
#[derive(Clone, Debug)]
pub struct ContractState {
    pub contract_id: String,
    pub storage: HashMap<String, Vec<u8>>, // Storage for key-value pairs
    pub get_state_result: Vec<u8>,
}

impl ContractState {
    // Constructor for ContractState
    pub fn new(contract_id: String) -> Self {
        Self {
            contract_id,
            storage: HashMap::new(), // Initialize storage
            get_state_result: Vec::new(),
        }
    }

    pub fn new_with_storage(contract_id: &str, storage: HashMap<String, Vec<u8>>) -> Self {
        Self {
            contract_id: contract_id.to_owned(),
            storage,
            get_state_result: Vec::new(),
        }
    }
}
