use std::sync::Arc;

use cesium_crypto::{
    errors::CryptoError,
    mldsa::da::{DABytes, DA_BYTE_LEN},
};
use cesium_standards::{BASE_TX_FEE, NATIVE_TOKEN_BYTES};
use dashmap::DashMap;
use tokio::{sync::Mutex, task::JoinError};

#[derive(Debug)]
pub enum InstructionError {
    NoInstructions,
    InvalidInstructionType,
    InstructionLengthIncongruency,
    ByteMismatch,
    InsufficientFunds,
    OutOfGas,
    CryptoError(CryptoError),
    JoinError(JoinError),
}

impl std::fmt::Display for InstructionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstructionError::NoInstructions => write!(f, "Transaction has no instructions"),
            InstructionError::InvalidInstructionType => write!(f, "Invalid instruction type"),
            InstructionError::InstructionLengthIncongruency => {
                write!(f, "Instruction length incongruency")
            }
            InstructionError::ByteMismatch => write!(f, "Byte mismatch"),
            InstructionError::InsufficientFunds => write!(f, "Insufficient funds"),
            InstructionError::OutOfGas => write!(f, "Out of gas"),
            InstructionError::CryptoError(e) => e.fmt(f),
            InstructionError::JoinError(e) => e.fmt(f),
        }
    }
}

impl From<CryptoError> for InstructionError {
    fn from(e: CryptoError) -> Self {
        InstructionError::CryptoError(e)
    }
}

impl From<JoinError> for InstructionError {
    fn from(e: JoinError) -> Self {
        InstructionError::JoinError(e)
    }
}

impl std::error::Error for InstructionError {}

macro_rules! bounds_check {
    ($bytes:expr, $pub_byte_len:expr) => {
        if $bytes.len() < $pub_byte_len {
            return Err(InstructionError::ByteMismatch);
        }
    };
}

#[derive(Debug, Clone, PartialEq)]
pub enum InstructionType {
    // Smart contracts
    ContractCall,
    ContractDeploy,
    // Currencies
    CurrencyTransfer,
    CurrencyCreate,
    CurrencyMint, // Only works if caller is the currency mint_authority
    CurrencyUpdate,
    // NFTs
    NFTBundleCreate,
    NFTBundleUpdate,
    NFTMint, // Bundle can be specified, but will only work if bundle update_authority is the same as caller
    NFTTransfer,
}

impl InstructionType {
    pub fn from_u8(value: u8) -> Option<InstructionType> {
        match value {
            0 => Some(InstructionType::ContractCall),
            1 => Some(InstructionType::ContractDeploy),
            2 => Some(InstructionType::CurrencyTransfer),
            3 => Some(InstructionType::CurrencyCreate),
            4 => Some(InstructionType::CurrencyMint),
            5 => Some(InstructionType::CurrencyUpdate),
            6 => Some(InstructionType::NFTBundleCreate),
            7 => Some(InstructionType::NFTBundleUpdate),
            8 => Some(InstructionType::NFTMint),
            9 => Some(InstructionType::NFTTransfer),
            _ => None,
        }
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            InstructionType::ContractCall => 0,
            InstructionType::ContractDeploy => 1,
            InstructionType::CurrencyTransfer => 2,
            InstructionType::CurrencyCreate => 3,
            InstructionType::CurrencyMint => 4,
            InstructionType::CurrencyUpdate => 5,
            InstructionType::NFTBundleCreate => 6,
            InstructionType::NFTBundleUpdate => 7,
            InstructionType::NFTMint => 8,
            InstructionType::NFTTransfer => 9,
        }
    }

    pub fn get_base_cost(&self) -> u128 {
        match self {
            InstructionType::ContractCall => 0,
            InstructionType::ContractDeploy => 0,
            InstructionType::CurrencyTransfer => 0,
            InstructionType::CurrencyCreate => 0,
            InstructionType::CurrencyMint => 0,
            InstructionType::CurrencyUpdate => 0,
            InstructionType::NFTBundleCreate => 0,
            InstructionType::NFTBundleUpdate => 0,
            InstructionType::NFTMint => 0,
            InstructionType::NFTTransfer => 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Instruction {
    pub instruction_type: InstructionType,
    pub data_length: u32,
    pub data: Vec<u8>,
}

impl Instruction {
    pub fn new(instruction_type: InstructionType, data: Vec<u8>) -> Instruction {
        Instruction {
            instruction_type,
            data_length: data.len() as u32,
            data,
        }
    }

    pub fn new_currency_transfer(
        currency: &[u8; DA_BYTE_LEN],
        amount: u128,
        recipient: &[u8; DA_BYTE_LEN],
    ) -> Instruction {
        let mut data = Vec::new();
        data.extend(currency.to_vec());
        data.extend(amount.to_le_bytes());
        data.extend(recipient.to_vec());

        Instruction::new(InstructionType::CurrencyTransfer, data)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.push(self.instruction_type.to_u8());
        bytes.extend(self.data_length.to_le_bytes());
        bytes.extend(self.data.clone());

        bytes
    }

    // This function should not really be called,
    // where this functionality is required it is done inline due to
    // performance and redundancy reasons.
    #[deprecated]
    pub fn from_bytes(bytes: &[u8]) -> Result<Instruction, InstructionError> {
        if bytes.len() < 5 {
            return Err(InstructionError::NoInstructions);
        }

        let mut offset = 0 as usize;

        // First get the type
        bounds_check!(bytes, offset + 1);
        let instr_type = InstructionType::from_u8(bytes[offset])
            .ok_or(InstructionError::InvalidInstructionType)?;
        offset += 1;

        // Get the length of the data
        bounds_check!(bytes, offset + 4);
        let data_len = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
        offset += 4;

        // Get the data
        bounds_check!(bytes, offset + data_len as usize);
        let data = bytes[offset..offset + data_len as usize].to_vec();

        Ok(Instruction {
            instruction_type: instr_type,
            data_length: data_len,
            data,
        })
    }
}

// TODO: Make this function prettier and easier to maintain
pub async fn execute_instructions(
    _signer_da: &DABytes,
    instructions: &[Instruction],
    provided_gas: u128,
) -> Result<(), InstructionError> {
    if provided_gas < BASE_TX_FEE {
        return Err(InstructionError::OutOfGas);
    }

    let mut can_run_concurrently = true;

    let mut has_currency_mint = false;
    let mut has_currency_transfer = false;

    // spender_balances holds the accurate balance of the spender with modifications made
    // across instructions
    let signer_balances: Arc<DashMap<[u8; DA_BYTE_LEN], u128>> = Arc::new(DashMap::new());
    // TODO: Get the balances from the disk
    let disk_balance: u128 = 0;
    if disk_balance < provided_gas {
        return Err(InstructionError::InsufficientFunds);
    }

    let mut used_gas: u128 = BASE_TX_FEE;
    for instr in instructions {
        // We do the cost calculate here to avoid having to wrap used_gas
        // in a mutex
        let i_cost = instr.instruction_type.get_base_cost();
        let balance = signer_balances.get(NATIVE_TOKEN_BYTES).unwrap();
        if *balance < i_cost {
            return Err(InstructionError::InsufficientFunds);
        } else if used_gas + i_cost > provided_gas {
            return Err(InstructionError::OutOfGas);
        }

        // Update the spender balances
        used_gas += i_cost;
        *signer_balances.get_mut(NATIVE_TOKEN_BYTES).unwrap() -= i_cost;

        if instr.instruction_type == InstructionType::ContractCall {
            can_run_concurrently = false;
        } else if instr.instruction_type == InstructionType::CurrencyMint {
            has_currency_mint = true;
            if has_currency_transfer {
                can_run_concurrently = false;
            }
        } else if instr.instruction_type == InstructionType::CurrencyTransfer {
            has_currency_transfer = true;
            if has_currency_mint {
                can_run_concurrently = false;
            }
        }
    }

    // recipient_delta_balances holds the delta balances of the recipients
    let recipient_delta_balances: Arc<DashMap<[u8; DA_BYTE_LEN], u128>> = Arc::new(DashMap::new());
    let used_gas: Arc<Mutex<u128>> = Arc::new(Mutex::new(used_gas)); // Used gas is updated by contract calls

    // The actual execution of the instructions
    if can_run_concurrently {
        // Run the instructions concurrently
        let mut futures = Vec::new();

        // Add tasks to the futures
        for instr in instructions {
            let signer_balances = Arc::clone(&signer_balances);
            let recipient_delta_balances = Arc::clone(&recipient_delta_balances);

            // Add as task to futures
            let instr = instr.clone(); // TODO: Optimize this
            let used_gas = Arc::clone(&used_gas);
            futures.push(tokio::spawn(async move {
                match instr.instruction_type {
                    InstructionType::ContractCall => {
                        // TODO: Implement contract call
                        let contract_cost = 0; // this will be provided by function call to runtime

                        let mut used_gas = used_gas.lock().await;
                        if *used_gas < contract_cost {
                            return Err(InstructionError::OutOfGas);
                        }
                        *used_gas -= contract_cost;
                    }
                    InstructionType::CurrencyTransfer => {
                        let mut offset = 0;
                        // Get the currency the signer is sending
                        bounds_check!(instr.data, DA_BYTE_LEN);
                        let currency: [u8; DA_BYTE_LEN] =
                            instr.data[offset..offset + DA_BYTE_LEN].try_into().unwrap();
                        offset += DA_BYTE_LEN;
                        // let currency_str = DisplayAddress::try_from_pk(&currency)?.as_str();

                        // Get the amount which is the next 16 bytes
                        bounds_check!(instr.data, offset + 16);
                        let amount = u128::from_le_bytes(
                            instr.data[offset..offset + 16].try_into().unwrap(),
                        );
                        offset += 16;

                        // Get the recipient which is the next PUB_KEY_LEN bytes
                        bounds_check!(instr.data, offset + DA_BYTE_LEN);
                        let recipient = instr.data[offset..offset + DA_BYTE_LEN].to_vec();

                        if let Some(spender) = signer_balances.get(&currency) {
                            if *spender < amount {
                                return Err(InstructionError::InsufficientFunds);
                            }
                        } else {
                            // TODO: Get current balance from disk
                            let disk_balance = 0;
                            if disk_balance < amount {
                                return Err(InstructionError::InsufficientFunds);
                            }

                            signer_balances.insert(currency, disk_balance);
                        }

                        // Update the spender balances
                        *signer_balances.get_mut(&currency).unwrap() -= amount;
                        *recipient_delta_balances
                            .entry(recipient.try_into().unwrap())
                            .or_insert(0) += amount;
                    }
                    _ => {}
                }
                Ok(())
            }));
        }

        // Wait for all tasks to finish
        for future in futures {
            future.await??;
        }
    } else {
        // Run the instructions sequentially
        for instr in instructions {
            match instr.instruction_type {
                InstructionType::ContractCall => {
                    // TODO: Implement contract call
                    let contract_cost = 0; // this will be provided by function call to runtime

                    let mut used_gas = used_gas.lock().await; // in theory this is not neccessary here, but used for concurrency block
                    if *used_gas < contract_cost {
                        return Err(InstructionError::OutOfGas);
                    }
                    *used_gas -= contract_cost;
                }
                InstructionType::CurrencyTransfer => {
                    let mut offset = 0;
                    // Get the currency the signer is sending
                    bounds_check!(instr.data, DA_BYTE_LEN);
                    let currency: [u8; DA_BYTE_LEN] =
                        instr.data[offset..offset + DA_BYTE_LEN].try_into().unwrap();
                    offset += DA_BYTE_LEN;
                    // let currency_str = DisplayAddress::try_from_pk(&currency)?.as_str();

                    // Get the amount which is the next 16 bytes
                    bounds_check!(instr.data, offset + 16);
                    let amount =
                        u128::from_le_bytes(instr.data[offset..offset + 16].try_into().unwrap());
                    offset += 16;

                    // Get the recipient which is the next PUB_KEY_LEN bytes
                    bounds_check!(instr.data, offset + DA_BYTE_LEN);
                    let recipient = instr.data[offset..offset + DA_BYTE_LEN].to_vec();

                    if let Some(spender) = signer_balances.get(&currency) {
                        if *spender < amount {
                            return Err(InstructionError::InsufficientFunds);
                        }
                    } else {
                        // TODO: Get current balance from disk
                        let disk_balance = 0;
                        if disk_balance < amount {
                            return Err(InstructionError::InsufficientFunds);
                        }

                        signer_balances.insert(currency, disk_balance);
                    }

                    // Update the spender balances
                    *signer_balances.get_mut(&currency).unwrap() -= amount;
                    *recipient_delta_balances
                        .entry(recipient.try_into().unwrap())
                        .or_insert(0) += amount;
                }
                _ => {}
            }
        }
    }

    // TODO: Update unconfirmed balances

    Ok(())
}

#[cfg(test)]
mod tests {
    use cesium_crypto::mldsa::da::DisplayAddress;

    use super::*;

    #[test]
    fn test_instruction() {
        let instruction = Instruction::new(InstructionType::CurrencyTransfer, vec![1, 2, 3]);

        let bytes = instruction.to_bytes();
        #[allow(deprecated)]
        let instruction2 = Instruction::from_bytes(&bytes).unwrap();

        assert_eq!(instruction.instruction_type, instruction2.instruction_type);
        assert_eq!(instruction.data, instruction2.data);
    }

    #[tokio::test]
    async fn test_execute_no_founds() {
        let signer = DisplayAddress::new_bytes();
        let currency = DisplayAddress::new_bytes();
        let recipient = DisplayAddress::new_bytes();

        let mut instructions = Vec::new();
        instructions.push(Instruction::new_currency_transfer(
            &currency, 1000, &recipient,
        ));

        let result = execute_instructions(&signer, &instructions, 1000).await;
        assert!(result
            .err()
            .unwrap()
            .to_string()
            .eq(&InstructionError::InsufficientFunds.to_string()));
    }

    #[tokio::test]
    async fn test_execute_insufficient_gas_upfront() {
        let signer = DisplayAddress::new_bytes();
        let currency = DisplayAddress::new_bytes();
        let recipient = DisplayAddress::new_bytes();

        let mut instructions = Vec::new();
        instructions.push(Instruction::new_currency_transfer(&currency, 0, &recipient));

        assert!(BASE_TX_FEE > 0);
        let result = execute_instructions(&signer, &instructions, 0).await;
        assert!(result
            .err()
            .unwrap()
            .to_string()
            .eq(&InstructionError::OutOfGas.to_string()));
    }
}
