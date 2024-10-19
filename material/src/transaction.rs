use crate::{instruction::Instruction, keys::KeyPair};

pub struct Transaction {
    pub instructions: Vec<Instruction>,
    pub signature: Option<Vec<u8>>,
}

impl Transaction {
    pub fn new() -> Transaction {
        Transaction {
            instructions: Vec::new(),
            signature: None,
        }
    }

    pub fn add_instruction(
        &mut self,
        instruction: Instruction,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // TODO: Validate instruction
        self.instructions.push(instruction);
        Ok(())
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        for instruction in &self.instructions {
            bytes.extend(instruction.to_bytes());
        }
        bytes
    }

    pub fn detached_hash(
        &mut self,
        kp: &KeyPair,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let message = self.to_bytes();

        let result = kp.digest(&message)?;
        println!("Tx siglen: {:?}", result.len());

        self.signature = Some(result);
        Ok(())
    }

    pub fn verify(
        &self,
        kp: KeyPair,
        signature: &[u8],
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        if self.signature.is_none() {
            return Err("Transaction is not signed".into());
        }

        let msg = self.to_bytes();
        Ok(kp.verify(&msg, signature)?)
    }
}
