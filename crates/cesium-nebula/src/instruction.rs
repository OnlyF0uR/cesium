#[derive(Debug, Clone)]
pub enum InstructionType {
    // Smart contracts
    ContractCall,
    ContractDeploy,
    // Currencies
    CurrencyTransfer,
    CurrencyCreate,
    CurrencyMint, // Only works if caller is the currency mint_authority
    CurrencyUpdate,
    CurrencyEnableStaking,
    CurrencyStake,
    CurrencyUnstake,
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
            6 => Some(InstructionType::CurrencyEnableStaking),
            7 => Some(InstructionType::CurrencyStake),
            8 => Some(InstructionType::CurrencyUnstake),
            9 => Some(InstructionType::NFTBundleCreate),
            10 => Some(InstructionType::NFTBundleUpdate),
            11 => Some(InstructionType::NFTMint),
            12 => Some(InstructionType::NFTTransfer),
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
            InstructionType::CurrencyEnableStaking => 6,
            InstructionType::CurrencyStake => 7,
            InstructionType::CurrencyUnstake => 8,
            InstructionType::NFTBundleCreate => 9,
            InstructionType::NFTBundleUpdate => 10,
            InstructionType::NFTMint => 11,
            InstructionType::NFTTransfer => 12,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Instruction {
    pub instruction_type: InstructionType,
    pub data: Vec<u8>,
}

impl Instruction {
    pub fn new(instruction_type: InstructionType, data: Vec<u8>) -> Instruction {
        Instruction {
            instruction_type,
            data,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(self.instruction_type.to_u8());
        bytes.extend(self.data.clone());

        bytes
    }

    pub fn from_bytes(
        bytes: &[u8],
    ) -> Result<Instruction, Box<dyn std::error::Error + Send + Sync>> {
        if bytes.len() < 1 {
            return Err("Instruction is empty".into());
        }

        let instruction_type =
            InstructionType::from_u8(bytes[0]).ok_or("Invalid instruction type")?;
        let data = bytes[1..].to_vec();

        Ok(Instruction {
            instruction_type,
            data,
        })
    }
}
