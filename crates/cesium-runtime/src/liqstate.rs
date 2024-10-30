// A oncecell in arc

use std::{collections::HashMap, sync::Arc};
use tokio::sync::{OnceCell, RwLock};

use crate::env::ContractEnv;

// NOTE: For now we will keep the state in memory,
// but normally it would be stored on disk when
// transaction is committed

pub struct LiqState {
    pub liquid_states: RwLock<HashMap<String, ContractEnv>>,
}

impl LiqState {
    async fn initialize() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let ls = LiqState {
            liquid_states: RwLock::new(HashMap::new()),
        };

        Ok(ls)
    }
}

static LS_INSTANCE: OnceCell<Arc<LiqState>> = OnceCell::const_new();

async fn get_instance() -> Arc<LiqState> {
    LS_INSTANCE
        .get_or_init(|| async {
            let ls = LiqState::initialize()
                .await
                .expect("Failed to initialize database");
            Arc::new(ls)
        })
        .await
        .clone()
}

pub async fn get_liquid_state(
    contract_id: &str,
) -> Result<ContractEnv, Box<dyn std::error::Error + Send + Sync>> {
    let instance = get_instance().await;

    let states = instance.liquid_states.read().await;
    let state = states.get(contract_id).unwrap().clone();

    Ok(state)
}

pub async fn update_liquid_state(
    contract_id: &str,
    contract_state: &ContractEnv,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let instance = get_instance().await;

    let mut states = instance.liquid_states.write().await;
    // Just so this function is used for the correct reasons,
    // and we might one to make some limitation checks
    // while updating the state in the future, to filter out
    // impossible state changes
    if !states.contains_key(contract_id) {
        return Err("Contract state does not exist".into());
    }

    states.insert(contract_id.to_string(), contract_state.clone());

    Ok(())
}

// Where liquid state becomes solid
pub async fn commit_state(
    contract_id: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let instance = get_instance().await;

    // Take liquid state
    let mut states = instance.liquid_states.write().await;
    let state = states.get(contract_id).unwrap().clone();
    states.remove(contract_id);

    println!("State: {:?}", state);

    // TODO: Right the persistent state to disk

    Ok(())
}

pub async fn load_state(
    contract_id: &str,
    caller_id: &str,
) -> Result<ContractEnv, Box<dyn std::error::Error + Send + Sync>> {
    // TODO: Load state from disk
    let state = ContractEnv::new(contract_id, caller_id);

    let instance = get_instance().await;

    let mut states = instance.liquid_states.write().await;
    states.insert(contract_id.to_string(), state.clone());

    Ok(state)
}
