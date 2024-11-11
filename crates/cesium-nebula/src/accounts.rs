use std::rc::Rc;

use cesium_crypto::{
    id::to_readable_id,
    keys::{PublicKeyBytes, PUB_BYTE_LEN},
};
use cesium_storage::{errors::StorageError, RocksDBStore};
use selenide_runtime::errors::RuntimeError;

macro_rules! bounds_check {
    ($bytes:expr, $pub_byte_len:expr) => {
        if $bytes.len() < $pub_byte_len {
            // TODO: Return an error instead of panicking
            panic!("Out of bounds data account bytes");
        }
    };
}

pub struct UserAccount {
    id: PublicKeyBytes,
    data_account_count: u32,
    data_account_ids: Rc<Vec<PublicKeyBytes>>,
}

impl UserAccount {
    #[must_use]
    pub fn new(id: PublicKeyBytes, data_account_ids: Rc<Vec<PublicKeyBytes>>) -> UserAccount {
        UserAccount {
            id,
            data_account_count: data_account_ids.len() as u32,
            data_account_ids,
        }
    }

    pub async fn from_id(id: PublicKeyBytes) -> Option<UserAccount> {
        let result = match RocksDBStore::instance().async_get(id.to_vec()).await {
            Ok(result) => match result {
                Some(bytes) => Some(UserAccount::from_bytes(&bytes)),
                None => None,
            },
            Err(e) => {
                eprintln!("Error getting user account: {:?}", e);
                return None;
            }
        };

        result
    }

    pub fn address(&self) -> String {
        to_readable_id(&self.id)
    }

    pub fn get_data_account(&self, id: &PublicKeyBytes) -> Option<&PublicKeyBytes> {
        self.data_account_ids.iter().find(|&da| da == id)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.id);
        bytes.extend_from_slice(&self.data_account_count.to_le_bytes());
        for id in self.data_account_ids.iter() {
            bytes.extend_from_slice(id);
        }
        bytes
    }

    pub async fn write(&self) -> Result<(), StorageError> {
        let bytes = self.to_bytes();
        RocksDBStore::instance()
            .async_put(self.id.to_vec(), bytes)
            .await
    }

    pub fn from_bytes(bytes: &[u8]) -> UserAccount {
        bounds_check!(bytes, PUB_BYTE_LEN);
        let id: [u8; PUB_BYTE_LEN] = bytes[0..PUB_BYTE_LEN].try_into().unwrap();
        let mut offset = PUB_BYTE_LEN;

        bounds_check!(bytes, offset + 4);
        let data_account_count = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
        offset = offset + 4;

        bounds_check!(bytes, offset + data_account_count as usize * PUB_BYTE_LEN);
        let mut data_account_ids = Vec::new();
        for i in 0..data_account_count {
            let start = offset + i as usize * PUB_BYTE_LEN;
            let end = start + PUB_BYTE_LEN;
            data_account_ids.push(bytes[start..end].try_into().unwrap());
        }

        UserAccount {
            id,
            data_account_count,
            data_account_ids: Rc::new(data_account_ids),
        }
    }
}

pub struct ContractAccount {
    id: PublicKeyBytes,
    state_account_len: u32,
    state_account_id: Option<PublicKeyBytes>,
    program_binary_len: u32,
    program_binary: Rc<Vec<u8>>,
}

impl ContractAccount {
    #[must_use]
    pub fn new(
        id: PublicKeyBytes,
        program_binary: Rc<Vec<u8>>,
        state_account_id: Option<PublicKeyBytes>,
    ) -> ContractAccount {
        let state_account_len = state_account_id.is_some() as u32;
        let program_binary_len = program_binary.len() as u32;
        ContractAccount {
            id,
            state_account_len,
            state_account_id,
            program_binary_len,
            program_binary,
        }
    }

    pub async fn from_id(id: PublicKeyBytes) -> Option<ContractAccount> {
        let result = match RocksDBStore::instance().async_get(id.to_vec()).await {
            Ok(result) => match result {
                Some(bytes) => Some(ContractAccount::from_bytes(&bytes)),
                None => None,
            },
            Err(e) => {
                eprintln!("Error getting contract account: {:?}", e);
                return None;
            }
        };

        result
    }

    pub fn address(&self) -> String {
        to_readable_id(&self.id)
    }

    pub fn get_state_account(&self) -> Option<&PublicKeyBytes> {
        self.state_account_id.as_ref()
    }

    pub fn execute(&self, _data: &[u8]) -> Result<(), RuntimeError> {
        println!("Executing contract with id: {}", to_readable_id(&self.id));
        println!("Binary: {:?}", self.program_binary);
        todo!()
    }

    pub async fn write(&self) -> Result<(), StorageError> {
        let bytes = self.to_bytes();
        RocksDBStore::instance()
            .async_put(self.id.to_vec(), bytes)
            .await
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.id);
        bytes.extend_from_slice(&self.state_account_len.to_le_bytes());
        if let Some(id) = &self.state_account_id {
            bytes.extend_from_slice(id);
        }
        bytes.extend_from_slice(&self.program_binary_len.to_le_bytes());
        bytes.extend_from_slice(&self.program_binary);
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> ContractAccount {
        bounds_check!(bytes, PUB_BYTE_LEN);
        let id: [u8; PUB_BYTE_LEN] = bytes[0..PUB_BYTE_LEN].try_into().unwrap();
        let mut offset = PUB_BYTE_LEN;

        bounds_check!(bytes, offset + 4);
        let state_account_len = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
        offset = offset + 4;

        bounds_check!(bytes, offset + PUB_BYTE_LEN);
        let state_account_id = if state_account_len > 0 {
            let id: [u8; PUB_BYTE_LEN] = bytes[offset..offset + PUB_BYTE_LEN].try_into().unwrap();
            offset = offset + PUB_BYTE_LEN;
            Some(id)
        } else {
            None
        };

        bounds_check!(bytes, offset + 4);
        let program_binary_len = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
        offset = offset + 4;

        let program_binary = Rc::new(bytes[offset..offset + program_binary_len as usize].to_vec());

        ContractAccount {
            id,
            state_account_len,
            state_account_id,
            program_binary_len,
            program_binary,
        }
    }
}

pub struct DataAccount {
    id: PublicKeyBytes,
    owner: PublicKeyBytes,
    updater: PublicKeyBytes,
    data_len: u32,
    data: Vec<u8>,
}

impl DataAccount {
    #[must_use]
    pub fn new(
        id: PublicKeyBytes,
        owner: PublicKeyBytes,
        updater: PublicKeyBytes,
        data: Vec<u8>,
    ) -> DataAccount {
        DataAccount {
            id,
            owner,
            updater,
            data_len: data.len() as u32,
            data,
        }
    }

    pub async fn from_id(id: PublicKeyBytes) -> Option<DataAccount> {
        let result = match RocksDBStore::instance().async_get(id.to_vec()).await {
            Ok(result) => match result {
                Some(bytes) => Some(DataAccount::from_bytes(&bytes)),
                None => None,
            },
            Err(e) => {
                eprintln!("Error getting data account: {:?}", e);
                return None;
            }
        };

        result
    }

    pub fn address(&self) -> String {
        to_readable_id(&self.id)
    }

    pub fn owner_address(&self) -> String {
        to_readable_id(&self.owner)
    }

    pub fn update_updater(&self) -> String {
        to_readable_id(&self.updater)
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub async fn write(&self) -> Result<(), StorageError> {
        let bytes = self.to_bytes();
        RocksDBStore::instance()
            .async_put(self.id.to_vec(), bytes)
            .await
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.id);
        bytes.extend_from_slice(&self.owner);
        bytes.extend_from_slice(&self.updater);
        bytes.extend_from_slice(&self.data_len.to_le_bytes());
        bytes.extend_from_slice(&self.data);
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> DataAccount {
        bounds_check!(bytes, PUB_BYTE_LEN);
        let id: [u8; PUB_BYTE_LEN] = bytes[0..PUB_BYTE_LEN].try_into().unwrap();
        let offset = PUB_BYTE_LEN;

        bounds_check!(bytes, offset + PUB_BYTE_LEN);
        let owner: [u8; PUB_BYTE_LEN] = bytes[offset..offset + PUB_BYTE_LEN].try_into().unwrap();
        let offset = offset + PUB_BYTE_LEN;

        bounds_check!(bytes, offset + PUB_BYTE_LEN);
        let updater: [u8; PUB_BYTE_LEN] = bytes[offset..offset + PUB_BYTE_LEN].try_into().unwrap();
        let offset = offset + PUB_BYTE_LEN;

        bounds_check!(bytes, offset + 4);
        let data_len = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
        let offset = offset + 4;

        bounds_check!(bytes, offset + data_len as usize);
        let data = bytes[offset..offset + data_len as usize].to_vec();

        DataAccount {
            id,
            owner,
            updater,
            data_len,
            data,
        }
    }
}

pub struct CurrencyAccount {
    id: PublicKeyBytes,
    owner: PublicKeyBytes,
    decimals: u8,
    minter_len: u32,
    minter: Option<PublicKeyBytes>,
    short_name_len: u32,
    short_name: String,
    long_name_len: u32,
    long_name: String,
}

impl CurrencyAccount {
    #[must_use]
    pub fn new(
        id: PublicKeyBytes,
        owner: PublicKeyBytes,
        short_name: String,
        long_name: String,
        decimals: u8,
        minter: Option<PublicKeyBytes>,
    ) -> CurrencyAccount {
        CurrencyAccount {
            id,
            owner,
            decimals,
            minter_len: minter.is_some() as u32,
            minter,
            short_name_len: short_name.len() as u32,
            short_name,
            long_name_len: long_name.len() as u32,
            long_name,
        }
    }

    pub fn from_id(_id: PublicKeyBytes) -> ContractAccount {
        todo!()
    }

    pub fn address(&self) -> String {
        to_readable_id(&self.id)
    }

    pub fn owner_address(&self) -> String {
        to_readable_id(&self.owner)
    }

    pub fn short_name(&self) -> &str {
        &self.short_name
    }

    pub fn long_name(&self) -> &str {
        &self.long_name
    }

    pub fn decimals(&self) -> u8 {
        self.decimals
    }

    pub fn minter_address(&self) -> Option<String> {
        self.minter.as_ref().map(|minter| to_readable_id(minter))
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.id);
        bytes.extend_from_slice(&self.owner);
        bytes.extend_from_slice(&self.decimals.to_le_bytes());
        bytes.extend_from_slice(&self.minter_len.to_le_bytes());
        if let Some(minter) = &self.minter {
            bytes.extend_from_slice(minter);
        }
        bytes.extend_from_slice(&self.short_name_len.to_le_bytes());
        bytes.extend_from_slice(self.short_name.as_bytes());
        bytes.extend_from_slice(&self.long_name_len.to_le_bytes());
        bytes.extend_from_slice(self.long_name.as_bytes());
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> CurrencyAccount {
        bounds_check!(bytes, PUB_BYTE_LEN);
        let id: [u8; PUB_BYTE_LEN] = bytes[0..PUB_BYTE_LEN].try_into().unwrap();
        let mut offset = PUB_BYTE_LEN;

        bounds_check!(bytes, offset + PUB_BYTE_LEN);
        let owner: [u8; PUB_BYTE_LEN] = bytes[offset..offset + PUB_BYTE_LEN].try_into().unwrap();
        offset = offset + PUB_BYTE_LEN;

        bounds_check!(bytes, offset + 1);
        let decimals = bytes[offset];
        offset = offset + 1;

        bounds_check!(bytes, offset + 4);
        let minter_len = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
        offset = offset + 4;

        let minter = if minter_len > 0 {
            bounds_check!(bytes, offset + PUB_BYTE_LEN);
            let minter: [u8; PUB_BYTE_LEN] =
                bytes[offset..offset + PUB_BYTE_LEN].try_into().unwrap();
            offset = offset + PUB_BYTE_LEN;
            Some(minter)
        } else {
            None
        };

        bounds_check!(bytes, offset + 4);
        let short_name_len = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
        offset = offset + 4;

        bounds_check!(bytes, offset + short_name_len as usize);
        let short_name =
            String::from_utf8(bytes[offset..offset + short_name_len as usize].to_vec()).unwrap();
        offset = offset + short_name_len as usize;

        bounds_check!(bytes, offset + 4);
        let long_name_len = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
        offset = offset + 4;

        bounds_check!(bytes, offset + long_name_len as usize);
        let long_name =
            String::from_utf8(bytes[offset..offset + long_name_len as usize].to_vec()).unwrap();

        CurrencyAccount {
            id,
            owner,
            decimals,
            minter_len,
            minter,
            short_name_len,
            short_name,
            long_name_len,
            long_name,
        }
    }
}

#[cfg(test)]
mod tests {
    use cesium_crypto::{id::generate_id_slice, keys::Account};

    use super::*;

    #[test]
    fn test_user_account() {
        let id: [u8; 48] = generate_id_slice();

        let d_id1 = generate_id_slice();
        let d_id2 = generate_id_slice();
        let data_account_ids = Rc::new(vec![d_id1, d_id2]);

        let user_account = UserAccount::new(id, data_account_ids.clone());
        assert_eq!(user_account.address(), to_readable_id(&id));

        let bytes = user_account.to_bytes();
        let user_account2 = UserAccount::from_bytes(&bytes);
        assert_eq!(user_account2.address(), user_account.address());
        assert_eq!(user_account2.data_account_ids, data_account_ids);
    }

    #[test]
    fn test_contract_account() {
        let id: [u8; 48] = generate_id_slice();
        let state_account_id = Some(generate_id_slice());
        let program_binary = Rc::new(vec![1, 2, 3, 4]);

        let contract_account = ContractAccount::new(id, program_binary.clone(), state_account_id);
        assert_eq!(contract_account.address(), to_readable_id(&id));

        let bytes = contract_account.to_bytes();
        let contract_account2 = ContractAccount::from_bytes(&bytes);
        assert_eq!(contract_account2.address(), contract_account.address());
        assert_eq!(contract_account2.program_binary, program_binary);
        assert_eq!(contract_account2.state_account_id, state_account_id);
    }

    #[test]
    fn test_data_account() {
        let id = generate_id_slice();
        let owner = *Account::create().to_public_key_bytes();
        let updater = *Account::create().to_public_key_bytes();
        let data = vec![1, 2, 3, 4];
        let data_account = DataAccount::new(id, owner, updater, data.clone());
        assert_eq!(data_account.address(), to_readable_id(&id));
        assert_eq!(data_account.owner_address(), to_readable_id(&owner));
        assert_eq!(data_account.update_updater(), to_readable_id(&updater));

        let bytes = data_account.to_bytes();
        let data_account2 = DataAccount::from_bytes(&bytes);
        assert_eq!(data_account2.address(), data_account.address());
        assert_eq!(data_account2.owner_address(), data_account.owner_address());
        assert_eq!(
            data_account2.update_updater(),
            data_account.update_updater()
        );
        assert_eq!(data_account2.data(), data.as_slice());
    }

    #[test]
    fn test_currency_account() {
        let id = generate_id_slice();
        let owner = *Account::create().to_public_key_bytes();
        let short_name = "ABC".to_string();
        let long_name = "Alpha Beta Charlie".to_string();
        let decimals = 2;
        let minter = Some(owner);
        let currency_account = CurrencyAccount::new(
            id,
            owner,
            short_name.clone(),
            long_name.clone(),
            decimals,
            minter,
        );
        assert_eq!(currency_account.address(), to_readable_id(&id));
        assert_eq!(currency_account.owner_address(), to_readable_id(&owner));
        assert_eq!(currency_account.short_name(), short_name);
        assert_eq!(currency_account.long_name(), long_name);
        assert_eq!(currency_account.decimals(), decimals);
        assert_eq!(
            currency_account.minter_address(),
            Some(to_readable_id(&owner))
        );

        let bytes = currency_account.to_bytes();
        let currency_account2 = CurrencyAccount::from_bytes(&bytes);
        assert_eq!(currency_account2.address(), currency_account.address());
        assert_eq!(
            currency_account2.owner_address(),
            currency_account.owner_address()
        );
        assert_eq!(
            currency_account2.short_name(),
            currency_account.short_name()
        );
        assert_eq!(currency_account2.long_name(), currency_account.long_name());
        assert_eq!(currency_account2.decimals(), currency_account.decimals());
        assert_eq!(
            currency_account2.minter_address(),
            currency_account.minter_address()
        );
    }

    #[tokio::test]
    async fn test_storage_user_account() {
        let account = Account::create();

        let id = account.to_public_key_bytes();
        let data_account_ids = Rc::new(vec![*Account::create().to_public_key_bytes()]);
        let user_account = UserAccount::new(*id, data_account_ids.clone());

        user_account.write().await.unwrap();

        let user_account2 = UserAccount::from_id(*id).await.unwrap();
        assert_eq!(user_account2.address(), user_account.address());
    }
}
