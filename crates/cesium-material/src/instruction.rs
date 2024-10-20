use crate::constants::NATIVE_TOKEN;
use crate::keys;
use crate::keys::PublicKeyBytes;

pub struct DataParameter {
    pub d: Vec<u8>, // the data
    pub l: usize,   // the length of the data
}

pub struct Instruction {
    pub proc_root: PublicKeyBytes, // the address of the program we aspire to call
    pub proc_index: u8,            // the index of the function we aspire to call
    pub proc_params: Vec<DataParameter>,
}

impl Instruction {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // Add the root address which contains the procedures
        bytes.extend(self.proc_root);
        // Add the index of the procedure we would like to call
        bytes.extend(self.proc_index.to_be_bytes());
        // Add the parameters of the procedure
        for data in &self.proc_params {
            bytes.extend(data.d.clone());
        }

        bytes
    }

    pub fn from_bytes(
        bytes: &[u8],
    ) -> Result<Instruction, Box<dyn std::error::Error + Send + Sync>> {
        if bytes.len() < 1 {
            return Err("Instruction is empty".into());
        }

        // Let's start by reading proc_root
        let mut proc_root = [0; 48];
        proc_root.copy_from_slice(&bytes[0..48]);

        // Next we read the proc_index
        let proc_index = bytes[48];

        // Next we read the proc_params
        let mut proc_params = Vec::new();
        let mut offset = 49; // 48 (root) + 1 (proc_index)
        while offset < bytes.len() {
            let length = bytes[offset] as usize;
            let data = bytes[offset + 1..offset + 1 + length].to_vec();
            proc_params.push(DataParameter { d: data, l: length });
            offset += 1 + length;
        }

        Ok(Instruction {
            proc_root,
            proc_index,
            proc_params,
        })
    }

    pub fn new_transfer_instruction(
        sender: &PublicKeyBytes,
        receivers: &Vec<PublicKeyBytes>,
        currencies: &Vec<PublicKeyBytes>,
        amounts: Vec<u128>,
    ) -> Result<Instruction, Box<dyn std::error::Error + Send + Sync>> {
        if sender.len() != 48 {
            return Err("Sender must be 48 bytes".into());
        }

        if amounts.len() != receivers.len() || amounts.len() != currencies.len() {
            return Err("Amounts must be equal to receivers".into());
        }

        for receiver in receivers {
            if receiver.len() != 48 {
                return Err("Receiver must be 48 bytes".into());
            }
        }

        for currency in currencies {
            if currency.len() != 48 {
                return Err("Currency must be 48 bytes".into());
            }
        }

        for amount in &amounts {
            if *amount == 0 {
                return Err("Amount must be greater than 0".into());
            }
        }

        let mut data: Vec<DataParameter> = Vec::with_capacity(4);
        // The first parameter is the sender public key
        let sender_param = DataParameter {
            d: sender.to_vec(),
            l: 48,
        };
        // The second parameter is the list of receivers public keys
        let receivers_param = DataParameter {
            d: receivers.concat(),
            l: receivers.len() * 48,
        };
        // The third parameter is the list of currencies public keys
        let currencies_param = DataParameter {
            d: currencies.concat(),
            l: currencies.len() * 48,
        };
        // The fourth parameter is the list of amounts
        let amounts_param = DataParameter {
            d: amounts
                .iter()
                .flat_map(|&amount| amount.to_be_bytes().to_vec())
                .collect(),
            l: amounts.len() * 8,
        };

        data.push(sender_param);
        data.push(receivers_param);
        data.push(currencies_param);
        data.push(amounts_param);

        let proc_root = keys::address_to_bytes(NATIVE_TOKEN)?;

        Ok(Instruction {
            proc_root,
            proc_index: 0,
            proc_params: data,
        })
    }

    pub fn new_custom_instruction(
        proc_root: PublicKeyBytes,
        proc_index: u8,
        proc_params: Vec<DataParameter>,
    ) -> Result<Instruction, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Instruction {
            proc_root,
            proc_index,
            proc_params,
        })
    }
}
