#[derive(Clone, Debug)]
pub struct ContractState {
    pub contract_id: String,
    pub caller_id: String,
    pub storage: Vec<Vec<u8>>, // Storage for key-value pairs
    pub storage_initialized: bool,
    pub get_state_result: Vec<u8>,
    pub get_address_result: Vec<u8>,
}

impl ContractState {
    // Constructor for ContractState
    pub fn new(contract_id: &str, caller_id: &str) -> Self {
        Self {
            contract_id: contract_id.to_owned(),
            caller_id: caller_id.to_owned(),
            storage: Vec::new(), // Initialize storage
            storage_initialized: false,
            get_state_result: Vec::new(),
            get_address_result: Vec::new(),
        }
    }

    pub fn new_with_storage(contract_id: &str, caller_id: &str, storage: Vec<Vec<u8>>) -> Self {
        Self {
            contract_id: contract_id.to_owned(),
            caller_id: caller_id.to_owned(),
            storage,
            storage_initialized: true,
            get_state_result: Vec::new(),
            get_address_result: Vec::new(),
        }
    }
}
