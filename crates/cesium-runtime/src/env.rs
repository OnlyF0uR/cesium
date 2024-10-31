use std::collections::HashMap;

// type for address
pub type AccountId = Vec<u8>;

#[derive(Clone, Debug)]
pub struct ContractState {
    pub initialized: bool,
    pub data: Vec<Vec<u8>>,
    pub committed: bool,
}

#[derive(Clone, Debug)]
pub struct AccountDataItem {
    pub owner: AccountId,       // owner of the data account
    pub update_auth: AccountId, // address of account that can update the data
    pub data: Vec<u8>,          // the actual data
}

#[derive(Clone, Debug)]
pub struct AccountData {
    pub data: HashMap<Vec<u8>, AccountDataItem>, // address-data pairs
    pub committed: bool,
}

#[derive(Clone, Debug)]
pub struct ContractEnv {
    pub contract_id: AccountId,
    pub caller_id: AccountId,

    pub state: ContractState,
    pub account_data: AccountData,

    pub mem_offset: u32,
}

impl ContractEnv {
    // Constructor for ContractEnv
    pub fn new(contract_id: &str, caller_id: &str) -> Self {
        let state = ContractState {
            initialized: false,
            data: Vec::new(),
            committed: false,
        };
        let account_data = AccountData {
            data: HashMap::new(),
            committed: false,
        };
        Self {
            contract_id: contract_id.as_bytes().to_vec(),
            caller_id: caller_id.as_bytes().to_vec(),
            state,
            account_data,
            mem_offset: 0,
        }
    }

    pub fn new_with_state(contract_id: &str, caller_id: &str, state_data: Vec<Vec<u8>>) -> Self {
        let state = ContractState {
            initialized: true,
            data: state_data,
            committed: false,
        };
        let account_data = AccountData {
            data: HashMap::new(),
            committed: false,
        };
        Self {
            contract_id: contract_id.as_bytes().to_vec(),
            caller_id: caller_id.as_bytes().to_vec(),
            state,
            account_data,
            mem_offset: 0,
        }
    }
}
