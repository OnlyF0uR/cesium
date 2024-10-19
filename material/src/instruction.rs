use crate::keys::PublicKeyBytes;

pub enum InstructionType {
    Transfer,
    Program,
}

pub struct DataParameter {
    pub d: Vec<u8>,
    pub l: usize,
}

pub struct Instruction {
    pub r#type: InstructionType,
    pub data: Vec<DataParameter>,
}

impl Instruction {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        match self.r#type {
            InstructionType::Transfer => {
                bytes.push(0);
            }
            InstructionType::Program => {
                bytes.push(1);
            }
        }

        for data in &self.data {
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

        let r#type = match bytes[0] {
            0 => InstructionType::Transfer,
            1 => InstructionType::Program,
            _ => return Err("Unknown instruction type".into()),
        };

        let mut data: Vec<DataParameter> = Vec::new();
        let mut offset = 1;
        while offset < bytes.len() {
            let l = bytes[offset] as usize;
            let d = bytes[offset + 1..offset + 1 + l].to_vec();
            data.push(DataParameter { d, l });
            offset += l + 1;
        }

        Ok(Instruction { r#type, data })
    }

    pub fn new_transfer_instruction(
        sender: PublicKeyBytes,
        receivers: Vec<PublicKeyBytes>,
        currencies: Vec<PublicKeyBytes>,
        amounts: Vec<u128>,
    ) -> Result<Instruction, Box<dyn std::error::Error + Send + Sync>> {
        if sender.len() != 48 {
            return Err("Sender must be 48 bytes".into());
        }

        if amounts.len() != receivers.len() || amounts.len() != currencies.len() {
            return Err("Amounts must be equal to receivers".into());
        }

        for receiver in &receivers {
            if receiver.len() != 48 {
                return Err("Receiver must be 48 bytes".into());
            }
        }

        for currency in &currencies {
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
        let sender_param = DataParameter { d: sender, l: 48 };
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

        Ok(Instruction {
            r#type: InstructionType::Transfer,
            data,
        })
    }
}
