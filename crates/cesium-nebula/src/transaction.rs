use cesium_crypto::keys::Account;

use crate::instruction::Instruction;

#[derive(Debug)]
pub struct Transaction {
    pub instructions: Vec<Instruction>,
    pub reserved_gas: u128,
    pub priority_fee: u128,
    pub digest: Option<Vec<u8>>,
}

impl Transaction {
    pub fn new(reserved_gas: u128, priority_fee: u128) -> Transaction {
        Transaction {
            instructions: Vec::new(),
            reserved_gas,
            priority_fee,
            digest: None,
        }
    }

    pub fn add_instruction(
        &mut self,
        instruction: Instruction,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.instructions.push(instruction);
        Ok(())
    }

    pub fn to_sig_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        for instruction in &self.instructions {
            bytes.extend(instruction.to_bytes());
        }
        bytes.extend(self.reserved_gas.to_le_bytes());
        bytes.extend(self.priority_fee.to_le_bytes());
        bytes
    }

    pub fn sign(&mut self, kp: &Account) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let message = self.to_sig_bytes();
        let result = kp.digest(&message)?;

        self.digest = Some(result);
        Ok(())
    }

    pub fn verify(&self, kp: &Account) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        if self.digest.is_none() {
            return Err("Transaction is not signed".into());
        }

        let msg = self.to_sig_bytes();
        Ok(kp.verify(&msg, &self.digest.as_ref().unwrap())?)
    }
}

#[cfg(test)]
mod tests {
    use crate::instruction::InstructionType;

    use super::*;

    #[test]
    fn test_transaction() {
        let acc = Account::create();

        let mut tx = Transaction::new(100, 10);
        let instruction = Instruction::new(InstructionType::CurrencyTransfer, vec![1, 2, 3]);
        tx.add_instruction(instruction).unwrap();

        tx.sign(&acc).unwrap();

        let valid = tx.verify(&acc);
        assert!(valid.unwrap());
    }
}
