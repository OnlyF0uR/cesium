use std::sync::{Arc, Mutex};

use wasmer::Memory;

// type for address
pub type AccountId = Vec<u8>;

#[derive(Clone, Debug)]
pub struct ContractState {
    pub values: Vec<Vec<u8>>,
}

impl ContractState {
    pub fn new() -> Self {
        Self { values: Vec::new() }
    }
}

#[derive(Clone, Debug)]
pub struct DataAccount {
    pub owner: AccountId,
    pub data: Vec<u8>,
    pub update_auth: Option<AccountId>,
}

#[derive(Clone, Debug)]
pub struct ContractDataAccounts {
    pub accounts: Vec<DataAccount>,
}

impl ContractDataAccounts {
    pub fn new() -> Self {
        Self {
            accounts: Vec::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ContractEnv {
    pub program_id: Arc<Mutex<String>>,
    pub caller_id: Arc<Mutex<String>>,
    pub memory: Option<Memory>,
    pub mem_offset: Arc<Mutex<u64>>,
    pub state: Arc<Mutex<ContractState>>,
    pub data_accounts: Arc<Mutex<ContractDataAccounts>>,
}

impl ContractEnv {
    pub fn new(
        program_id: &str,
        caller_id: &str,
        state: Arc<Mutex<ContractState>>,
        data_accounts: Arc<Mutex<ContractDataAccounts>>,
        memory_offset: u64,
    ) -> Self {
        Self {
            program_id: Arc::new(Mutex::new(program_id.to_string())),
            caller_id: Arc::new(Mutex::new(caller_id.to_string())),
            memory: None,
            mem_offset: Arc::new(Mutex::new(memory_offset)),
            state,
            data_accounts,
        }
    }
}
