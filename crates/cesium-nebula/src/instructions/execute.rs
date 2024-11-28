use std::sync::Arc;

use cesium_crypto::mldsa::da::{DABytes, DA_BYTE_LEN};
use cesium_standards::{BASE_TX_FEE, NATIVE_TOKEN_BYTES};
use dashmap::DashMap;
use tokio::sync::Mutex;

use super::{
    errors::InstructionError,
    instruction::{Instruction, InstructionType},
};

macro_rules! bounds_check {
    ($bytes:expr, $pub_byte_len:expr) => {
        if $bytes.len() < $pub_byte_len {
            return Err(InstructionError::ByteMismatch);
        }
    };
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
    // spender_balances holds the accurate balance of the spender with modifications made
    // across instructions
    let signer_balances: Arc<DashMap<[u8; DA_BYTE_LEN], u128>> = Arc::new(DashMap::new());
    // TODO: Get the balances from the disk
    let disk_balance: u128 = 0;
    if disk_balance < provided_gas {
        return Err(InstructionError::InsufficientFunds);
    }

    let mut used_gas: u128 = BASE_TX_FEE;
    let can_run_concurrently =
        preprocess_instructions(&signer_balances, &mut used_gas, provided_gas, instructions)?;

    // recipient_delta_balances holds the delta balances of the recipients
    let recipient_delta_balances: Arc<DashMap<[u8; DA_BYTE_LEN], u128>> = Arc::new(DashMap::new());
    let used_gas: Arc<Mutex<u128>> = Arc::new(Mutex::new(used_gas)); // Used gas is updated by contract calls

    // The actual execution of the instructions
    if can_run_concurrently {
        // Run the instructions concurrently
        let mut futures: Vec<tokio::task::JoinHandle<Result<(), InstructionError>>> = Vec::new();

        // Add tasks to the futures
        for instr in instructions {
            let signer_balances = Arc::clone(&signer_balances);
            let recipient_delta_balances = Arc::clone(&recipient_delta_balances);

            // Add as task to futures
            let instr = instr.clone(); // TODO: Optimize this
            let used_gas = Arc::clone(&used_gas);
            futures.push(tokio::spawn(async move {
                execute_instruction(
                    &signer_balances,
                    &recipient_delta_balances,
                    &used_gas,
                    &instr,
                )
                .await?;

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
            execute_instruction(
                &signer_balances,
                &recipient_delta_balances,
                &used_gas,
                &instr,
            )
            .await?;
        }
    }

    // TODO: Update unconfirmed balances
    println!("{:?}", signer_balances);
    println!("{:?}", recipient_delta_balances);

    Ok(())
}

fn preprocess_instructions(
    signer_balances: &Arc<DashMap<[u8; DA_BYTE_LEN], u128>>,
    used_gas: &mut u128,
    provided_gas: u128,
    instructions: &[Instruction],
) -> Result<bool, InstructionError> {
    let mut can_run_concurrently = true;
    let mut has_currency_mint = false;
    let mut has_currency_transfer = false;

    for instr in instructions {
        // We do the cost calculate here to avoid having to wrap used_gas
        // in a mutex
        let i_cost = instr.instruction_type.get_base_cost();
        let balance = signer_balances.get(NATIVE_TOKEN_BYTES).unwrap();
        if *balance < i_cost {
            return Err(InstructionError::InsufficientFunds);
        } else if *used_gas + i_cost > provided_gas {
            return Err(InstructionError::OutOfGas);
        }

        // Update the spender balances
        *used_gas += i_cost;
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

    Ok(can_run_concurrently)
}

async fn execute_instruction(
    signer_balances: &Arc<DashMap<[u8; DA_BYTE_LEN], u128>>,
    recipient_delta_balances: &Arc<DashMap<[u8; DA_BYTE_LEN], u128>>,
    used_gas: &Arc<Mutex<u128>>,
    instr: &Instruction,
) -> Result<(), InstructionError> {
    match instr.instruction_type {
        InstructionType::ContractCall => {
            contract_call(&used_gas).await?;
        }
        InstructionType::ContractDeploy => {
            contract_deploy().await?;
        }
        InstructionType::CurrencyTransfer => {
            currency_transfer(&signer_balances, &recipient_delta_balances, &instr.data).await?;
        }
        InstructionType::CurrencyCreate => {
            currency_create().await?;
        }
        InstructionType::CurrencyMint => {
            currency_mint().await?;
        }
        InstructionType::CurrencyUpdate => {
            currency_update().await?;
        }
        InstructionType::NFTBundleCreate => {
            nft_bundle_create().await?;
        }
        InstructionType::NFTBundleUpdate => {
            nft_bundle_update().await?;
        }
        InstructionType::NFTMint => {
            nft_mint().await?;
        }
        InstructionType::NFTTransfer => {
            nft_transfer().await?;
        }
    }

    Ok(())
}

async fn contract_call(used_gas: &Arc<Mutex<u128>>) -> Result<(), InstructionError> {
    // TODO: Implement contract call
    let contract_cost = 0; // this will be provided by function call to runtime

    let mut used_gas = used_gas.lock().await; // in theory this is not neccessary here, but used for concurrency block
    if *used_gas < contract_cost {
        return Err(InstructionError::OutOfGas);
    }
    *used_gas -= contract_cost;

    Ok(())
}

async fn contract_deploy() -> Result<(), InstructionError> {
    todo!()
}

async fn currency_transfer(
    signer_balances: &Arc<DashMap<[u8; DA_BYTE_LEN], u128>>,
    recipient_delta_balances: &Arc<DashMap<[u8; DA_BYTE_LEN], u128>>,
    instr_data: &[u8],
) -> Result<(), InstructionError> {
    let mut offset = 0;
    // Get the currency the signer is sending
    bounds_check!(instr_data, DA_BYTE_LEN);
    let currency: [u8; DA_BYTE_LEN] = instr_data[offset..offset + DA_BYTE_LEN].try_into().unwrap();
    offset += DA_BYTE_LEN;
    // let currency_str = DisplayAddress::try_from_pk(&currency)?.as_str();

    // Get the amount which is the next 16 bytes
    bounds_check!(instr_data, offset + 16);
    let amount = u128::from_le_bytes(instr_data[offset..offset + 16].try_into().unwrap());
    offset += 16;

    // Get the recipient which is the next PUB_KEY_LEN bytes
    bounds_check!(instr_data, offset + DA_BYTE_LEN);
    let recipient = instr_data[offset..offset + DA_BYTE_LEN].to_vec();

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

    Ok(())
}

async fn currency_create() -> Result<(), InstructionError> {
    todo!()
}

async fn currency_mint() -> Result<(), InstructionError> {
    todo!()
}

async fn currency_update() -> Result<(), InstructionError> {
    todo!()
}

async fn nft_bundle_create() -> Result<(), InstructionError> {
    todo!()
}

async fn nft_bundle_update() -> Result<(), InstructionError> {
    todo!()
}

async fn nft_mint() -> Result<(), InstructionError> {
    todo!()
}

async fn nft_transfer() -> Result<(), InstructionError> {
    todo!()
}

#[cfg(test)]
mod tests {
    use cesium_crypto::mldsa::da::DisplayAddress;

    use super::*;

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
