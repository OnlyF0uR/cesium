use std::rc::Rc;

use cesium_crypto::{id::to_readable_id, keys::PublicKeyBytes};
use selenide_runtime::errors::RuntimeError;

// TODO: Actually do include some useful creation and
// retrieval processes for these accounts

pub struct UserAccount {
    id: PublicKeyBytes,
    currency_account_ids: Rc<Vec<PublicKeyBytes>>,
    data_account_ids: Rc<Vec<PublicKeyBytes>>,
}

impl UserAccount {
    #[must_use]
    pub fn new(
        id: PublicKeyBytes,
        currency_account_ids: Rc<Vec<PublicKeyBytes>>,
        data_account_ids: Rc<Vec<PublicKeyBytes>>,
    ) -> UserAccount {
        UserAccount {
            id,
            currency_account_ids,
            data_account_ids,
        }
    }

    pub fn from_id(_id: PublicKeyBytes) -> ContractAccount {
        todo!()
    }

    pub fn address(&self) -> String {
        to_readable_id(&self.id)
    }

    pub fn get_currency_account(&self, id: &PublicKeyBytes) -> Option<&PublicKeyBytes> {
        self.currency_account_ids.iter().find(|&ca| ca == id)
    }

    pub fn get_data_account(&self, id: &PublicKeyBytes) -> Option<&PublicKeyBytes> {
        self.data_account_ids.iter().find(|&da| da == id)
    }
}

pub struct ContractAccount {
    id: PublicKeyBytes,
    program_binary: Rc<Vec<u8>>,
    state_account_id: Option<PublicKeyBytes>,
}

impl ContractAccount {
    #[must_use]
    pub fn new(
        id: PublicKeyBytes,
        program_binary: Rc<Vec<u8>>,
        state_account_id: Option<PublicKeyBytes>,
    ) -> ContractAccount {
        ContractAccount {
            id,
            program_binary,
            state_account_id,
        }
    }

    pub fn from_id(_id: PublicKeyBytes) -> ContractAccount {
        todo!()
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
}

pub struct CurrencyAccount {
    id: PublicKeyBytes,
    owner: PublicKeyBytes,
    short_name: String,
    long_name: String,
    decimals: u8,
    mint_authority: Option<PublicKeyBytes>,
}

impl CurrencyAccount {
    #[must_use]
    pub fn new(
        id: PublicKeyBytes,
        owner: PublicKeyBytes,
        mint_authority: Option<PublicKeyBytes>,
        short_name: String,
        long_name: String,
        decimals: u8,
    ) -> CurrencyAccount {
        CurrencyAccount {
            id,
            owner,
            short_name,
            long_name,
            decimals,
            mint_authority,
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

    pub fn mint_authority_address(&self) -> Option<String> {
        self.mint_authority.as_ref().map(|pk| to_readable_id(pk))
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
}

pub struct DataAccount {
    id: PublicKeyBytes,
    owner: PublicKeyBytes,
    update_authority: PublicKeyBytes,
    data: Vec<u8>,
}

impl DataAccount {
    #[must_use]
    pub fn new(
        id: PublicKeyBytes,
        owner: PublicKeyBytes,
        update_authority: PublicKeyBytes,
        data: Vec<u8>,
    ) -> DataAccount {
        DataAccount {
            id,
            owner,
            update_authority,
            data,
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

    pub fn update_authority_address(&self) -> String {
        to_readable_id(&self.update_authority)
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }
}
