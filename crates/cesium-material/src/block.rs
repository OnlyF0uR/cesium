use rand::Rng;

use crate::{
    constants::MAX_BLOCK_TRANSACTIONS,
    keys::{KeyPair, PublicKeyBytes},
    transaction::Transaction,
};

pub struct Block {
    pub index: u64,
    pub previous_hash: Vec<u8>, // length of message + 35664
    pub nonce: u128,
    pub transactions: Vec<Transaction>,
    pub validator_key: PublicKeyBytes,
    pub signature: Option<Vec<u8>>,
}

impl Block {
    pub fn new(
        index: u64,
        validator_kp: &KeyPair,
        previous_hash: Vec<u8>,
    ) -> Result<Block, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Block {
            index,
            previous_hash,
            nonce: generate_nonce(),
            transactions: Vec::new(),
            validator_key: *validator_kp.to_public_key_bytes(),
            signature: None,
        })
    }

    pub fn add_transaction(
        &mut self,
        transaction: Transaction,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if self.transactions.len() >= MAX_BLOCK_TRANSACTIONS {
            return Err("Block is full".into());
        }

        if transaction.signature.is_none() {
            return Err("Transaction is not signed".into());
        }

        self.transactions.push(transaction);
        Ok(())
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(&self.index.to_be_bytes());
        bytes.extend(&self.previous_hash);
        bytes.extend(&self.nonce.to_be_bytes());
        bytes.extend(&self.validator_key);
        for transaction in &self.transactions {
            bytes.extend(transaction.to_bytes());
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
        kp: &KeyPair,
        signature: &[u8],
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        if self.signature.is_none() {
            return Err("Transaction is not signed".into());
        }

        let msg = self.to_bytes();
        Ok(kp.verify(&msg, signature)?)
    }

    pub fn derive_next(
        previous: &Block,
        validator_kp: &KeyPair,
    ) -> Result<Block, Box<dyn std::error::Error + Send + Sync>> {
        let mut block = Block::new(previous.index + 1, validator_kp, previous.to_bytes())?;
        block.detached_hash(validator_kp)?;
        Ok(block)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        constants::{NATIVE_DECIMALS, NATIVE_TOKEN},
        instruction::Instruction,
        keys::{self, KeyPair, SIG_BYTE_LEN},
    };

    #[test]
    fn test_block() {
        let sender_kp = KeyPair::create();
        let receiver_kp = KeyPair::create();

        let currency = keys::address_to_bytes(NATIVE_TOKEN).unwrap();
        let amount: u128 = (10000 * (NATIVE_DECIMALS as u128)).into();

        let instr: Instruction = Instruction::new_transfer_instruction(
            sender_kp.to_public_key_bytes(),
            &vec![receiver_kp.to_public_key_bytes().to_owned()],
            &vec![currency],
            vec![amount],
        )
        .unwrap();

        let mut tx = Transaction::new();
        tx.add_instruction(instr).unwrap();
        tx.detached_hash(&sender_kp).unwrap();

        // Vec null hash
        let null_hash = vec![0; SIG_BYTE_LEN];

        let mut block = Block::new(0, &sender_kp, null_hash).unwrap();
        block.add_transaction(tx).unwrap();
        block.detached_hash(&sender_kp).unwrap();

        let sig = block.signature.clone().unwrap();

        let verified = block.verify(&sender_kp, &sig).unwrap();
        assert!(verified);
    }
}

fn generate_nonce() -> u128 {
    let mut rng = rand::thread_rng();
    rng.gen()
}
