use cesium_crypto::keys::{PublicKeyBytes, PUB_BYTE_LEN};

pub struct CurrencyAmountMetadata {
    currency: PublicKeyBytes,
    amount: u128,
}

macro_rules! bounds_check {
    ($bytes:expr, $pub_byte_len:expr) => {
        if $bytes.len() < $pub_byte_len {
            // TODO: Return an error instead of panicking
            panic!("Out of bounds currency metadata bytes");
        }
    };
}

impl CurrencyAmountMetadata {
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut offset = 0;
        bounds_check!(bytes, offset + PUB_BYTE_LEN);
        let currency = bytes[offset..offset + PUB_BYTE_LEN].try_into()?;
        offset += PUB_BYTE_LEN;

        bounds_check!(bytes, offset + 16);
        let amount = u128::from_le_bytes(bytes[offset..offset + 16].try_into()?);

        Ok(Self { currency, amount })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.currency);
        bytes.extend_from_slice(&self.amount.to_le_bytes());
        bytes
    }
}
