#[derive(Debug)]
pub enum InstructionError {
    NoInstructions,
    InvalidInstructionType,
    InstructionLengthIncongruency,
    ByteMismatch,
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
        }
    }
}

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

#[cfg(test)]
mod tests {
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
}
