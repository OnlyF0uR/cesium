use std::sync::Arc;

use cesium_material::keys::PUB_BYTE_LEN;
use rand::RngCore;
use rocksdb::{Options, WriteBatch, DB};
use serde::{Deserialize, Serialize};
use sha3::Digest;

use crate::data::DataObject;

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    pub address: String,
    pub program_binary: Option<Vec<u8>>,
    pub data_account_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DataAccount {
    pub id: String,
    pub data: Vec<DataObject>,
}

pub struct DataAccountManager {
    db: Arc<DB>,
}

impl DataAccountManager {
    pub fn new(path: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut options = Options::default();
        options.create_if_missing(true);
        options.increase_parallelism(2);
        options.set_max_background_jobs(2);
        options.set_write_buffer_size(64 * 1024 * 1024); // 64MB
        options.set_max_write_buffer_number(3);

        let db = Arc::new(DB::open(&options, path)?);
        Ok(Self { db })
    }

    pub fn create_data_account(
        &self,
        user_address: &str,
        obj: &[DataObject],
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Generate ID using thread-local RNG for better performance
        let mut rng = rand::thread_rng();
        let mut id = [0u8; 32];
        rng.fill_bytes(&mut id);

        let mut hasher = sha3::Sha3_384::new();
        hasher.update(id);
        let result = hasher.finalize().to_vec();

        // We want to ensure that our hash is the size of
        // a public key, so we can use it as an ID, now a
        // sha3_384 hash is 48 bytes, so that works.
        if result.len() != PUB_BYTE_LEN {
            panic!("Invalid public key length");
        }

        let id = bs58::encode(result).into_string();
        let account_key = format!("account:{}", user_address);

        // Use a write batch for atomic operations
        let mut batch = WriteBatch::default();

        // Get current account or create new one
        let mut account = match self.db.get(account_key.as_bytes())? {
            Some(data) => bincode::deserialize(&data)?,
            None => Account {
                address: user_address.to_string(),
                program_binary: None,
                data_account_ids: Vec::with_capacity(1),
            },
        };

        // Create new data account
        let new_data_account = DataAccount {
            id: id.clone(),
            data: obj.to_vec(),
        };

        // Prepare batch operations
        let data_account_key = format!("data_account:{}", id);
        batch.put(
            data_account_key.as_bytes(),
            bincode::serialize(&new_data_account)?,
        );

        account.data_account_ids.push(id.clone());
        batch.put(account_key.as_bytes(), bincode::serialize(&account)?);

        // Atomic write
        self.db.write(batch)?;

        Ok(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cesium_material::keys::KeyPair;
    use tempfile::TempDir;

    #[test]
    fn test_create_data_account() {
        let temp_dir = TempDir::new().unwrap();
        let manager = DataAccountManager::new(temp_dir.path().to_str().unwrap()).unwrap();

        let kp = KeyPair::create();
        let address = kp.to_public_key_readable();

        let data_object = DataObject {
            type_id: *KeyPair::create().to_public_key_bytes(),
            data: vec![0x01, 0x02, 0x03],
        };

        let id = manager
            .create_data_account(&address, &[data_object])
            .unwrap();
        assert!(!id.is_empty());
    }
}
