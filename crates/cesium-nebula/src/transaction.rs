use std::time::SystemTime;

use cesium_crypto::{
    errors::CryptoError,
    mldsa::{
        da::DisplayAddress,
        keypair::{SignerPair, VerifierPair, ViewOperations},
        PublicKeyBytes, PUB_BYTE_LEN,
    },
};

use crate::instruction::{Instruction, InstructionError, InstructionType};

#[derive(Debug)]
pub enum TransactionError {
    NotSigned,
    InstructionError(InstructionError),
    ByteMismatch,
    CryptoError(CryptoError),
    InvalidSignature,
}

impl std::fmt::Display for TransactionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionError::NotSigned => write!(f, "Transaction is not signed"),
            TransactionError::InstructionError(e) => e.fmt(f),
            TransactionError::ByteMismatch => write!(f, "Byte mismatch"),
            TransactionError::CryptoError(e) => e.fmt(f),
            TransactionError::InvalidSignature => write!(f, "Invalid signature"),
        }
    }
}

impl From<InstructionError> for TransactionError {
    fn from(e: InstructionError) -> Self {
        TransactionError::InstructionError(e)
    }
}

impl From<CryptoError> for TransactionError {
    fn from(e: CryptoError) -> Self {
        TransactionError::CryptoError(e)
    }
}

macro_rules! bounds_check {
    ($bytes:expr, $pub_byte_len:expr) => {
        if $bytes.len() < $pub_byte_len {
            return Err(TransactionError::ByteMismatch);
        }
    };
}

#[derive(Debug)]
pub struct Transaction {
    pub instructions_count: u64,
    pub instructions: Vec<Instruction>,
    pub reserved_gas: u128,
    pub priority_fee: u128,
    pub timestamp: u64,
    pub signer: Option<PublicKeyBytes>,
    pub digest: Option<Vec<u8>>,
}

impl Transaction {
    #[must_use]
    pub fn new(reserved_gas: u128, priority_fee: u128) -> Transaction {
        Transaction {
            instructions_count: 0,
            instructions: Vec::new(),
            reserved_gas,
            priority_fee,
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            signer: None,
            digest: None,
        }
    }

    pub fn add_instruction(&mut self, instruction: Instruction) -> Result<(), TransactionError> {
        self.instructions_count += 1;
        self.instructions.push(instruction);
        Ok(())
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, TransactionError> {
        if !self.is_signed() {
            return Err(TransactionError::NotSigned);
        }

        let mut bytes = Vec::new();
        bytes.extend(self.instructions_count.to_le_bytes());
        for instruction in &self.instructions {
            bytes.extend(instruction.to_bytes());
        }
        bytes.extend(self.reserved_gas.to_le_bytes());
        bytes.extend(self.priority_fee.to_le_bytes());
        bytes.extend(self.timestamp.to_le_bytes());
        bytes.extend(self.signer.unwrap());
        bytes.extend(self.digest.as_ref().unwrap());
        Ok(bytes)
    }

    pub fn signer_da(&self) -> Option<String> {
        self.signer.map(|s| DisplayAddress::from_pk(&s).as_str())
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, TransactionError> {
        let mut offset = 0;
        bounds_check!(bytes, offset + 8);
        let instructions_count = u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap());
        offset += 8;

        let mut instructions = Vec::new();
        // We will decode and construct the instruction here because we need to know the length
        // anyway which involves reading 1 + 4 bytes into it, so we might as well read the last part (data)
        // too, to prevent repeating ourselves for the first 2 reads.
        for _ in 0..instructions_count {
            // We now need to know how long each transaction is, we can do this by first skipping
            // the type of the instruction which is 1
            bounds_check!(bytes, offset + 1);
            let instr_type = InstructionType::from_u8(bytes[offset])
                .ok_or(InstructionError::InvalidInstructionType)?;
            offset += 1;
            // Then we are onto the u32 data length which is the length of the data that follows
            bounds_check!(bytes, offset + 4);
            let data_len =
                u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
            offset += 4;
            // Now we know how much instruction data we need to read for this instruction
            let data = bytes[offset..offset + data_len].to_vec();
            offset += data_len;

            let instr = Instruction {
                instruction_type: instr_type,
                data_length: data_len as u32,
                data,
            };
            instructions.push(instr);
        }

        // We now read the reserved gas which is a u128 (16 bytes)
        bounds_check!(bytes, offset + 16);
        let reserved_gas = u128::from_le_bytes(bytes[offset..offset + 16].try_into().unwrap());
        offset += 16;

        // We now read the priority fee which is a u128 (16 bytes)
        bounds_check!(bytes, offset + 16);
        let priority_fee = u128::from_le_bytes(bytes[offset..offset + 16].try_into().unwrap());
        offset += 16;

        // We now read the timestamp which is a u64 (8 bytes)
        bounds_check!(bytes, offset + 8);
        let timestamp = u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap());
        offset += 8;

        let mut signer: Option<PublicKeyBytes> = None;
        let mut digest = None;

        // Check if the transaction is signed
        if bytes.len() - offset >= PUB_BYTE_LEN {
            signer = Some(bytes[offset..offset + PUB_BYTE_LEN].try_into().unwrap());
            offset += PUB_BYTE_LEN;

            digest = Some(bytes[offset..].to_vec());
        }

        // We have read all the bytes, we can now construct the transaction
        Ok(Transaction {
            instructions_count,
            instructions,
            reserved_gas,
            priority_fee,
            timestamp,
            signer,
            digest,
        })
    }

    pub fn is_signed(&self) -> bool {
        self.signer.is_some() && self.digest.is_some()
    }

    pub fn create_id(&self) -> Result<String, TransactionError> {
        let id = DisplayAddress::new_from_seed(self.timestamp.to_le_bytes().as_ref());
        Ok(id.as_str())
    }

    pub fn to_sig_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.instructions_count.to_le_bytes());
        for instruction in &self.instructions {
            bytes.extend(instruction.to_bytes());
        }
        bytes.extend(self.reserved_gas.to_le_bytes());
        bytes.extend(self.priority_fee.to_le_bytes());
        bytes.extend(self.timestamp.to_le_bytes());

        bytes
    }

    pub fn sign(&mut self, kp: &SignerPair) -> Result<(), TransactionError> {
        let message = self.to_sig_bytes();
        let result = kp.sign(&message);

        self.signer = Some(*kp.pub_key_bytes());
        self.digest = Some(result);
        Ok(())
    }

    pub fn verify_ext(&self, kp: &VerifierPair) -> Result<bool, TransactionError> {
        if self.digest.is_none() {
            return Err(TransactionError::NotSigned);
        }

        let msg = self.to_sig_bytes();
        Ok(kp.verify(&msg, &self.digest.as_ref().unwrap())?)
    }

    pub fn verify(&self) -> Result<bool, TransactionError> {
        if self.signer.is_none() || self.digest.is_none() {
            return Err(TransactionError::NotSigned);
        }

        let kp = VerifierPair::from_bytes(&self.signer.unwrap())?;
        let msg = self.to_sig_bytes();
        Ok(kp.verify(&msg, &self.digest.as_ref().unwrap()).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use crate::instruction::InstructionType;

    use super::*;

    #[test]
    fn test_tx() {
        let acc = SignerPair::create();

        let mut tx = Transaction::new(100, 10);
        let instruction = Instruction::new(InstructionType::CurrencyTransfer, vec![1, 2, 3]);
        tx.add_instruction(instruction).unwrap();

        tx.sign(&acc).unwrap();

        let valid = tx.verify();
        assert!(valid.unwrap());
    }

    #[test]
    fn test_tx_bytes() {
        let acc = SignerPair::create();

        let mut tx = Transaction::new(100, 10);
        let instruction = Instruction::new(InstructionType::CurrencyTransfer, vec![1, 2, 3]);
        tx.add_instruction(instruction).unwrap();

        tx.sign(&acc).unwrap();

        let bytes = tx.to_bytes().unwrap();
        let tx2 = Transaction::from_bytes(&bytes).unwrap();

        assert_eq!(tx.instructions_count, tx2.instructions_count);
        assert_eq!(tx.instructions, tx2.instructions);
        assert_eq!(tx.reserved_gas, tx2.reserved_gas);
        assert_eq!(tx.priority_fee, tx2.priority_fee);
        assert_eq!(tx.timestamp, tx2.timestamp);
        assert_eq!(tx.signer, tx2.signer);
        assert_eq!(tx.digest, tx2.digest);
    }
}
