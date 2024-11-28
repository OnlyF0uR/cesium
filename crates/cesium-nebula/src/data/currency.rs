use cesium_crypto::mldsa::da::{DABytes, DA_BYTE_LEN};

macro_rules! bounds_check {
    ($bytes:expr, $pub_byte_len:expr) => {
        if $bytes.len() < $pub_byte_len {
            // TODO: Return an error instead of panicking
            panic!("Out of bounds currency metadata bytes");
        }
    };
}

pub struct CurrencyHolderData {
    currency: DABytes,
    amount: u128,
}

impl CurrencyHolderData {
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut offset = 0;
        bounds_check!(bytes, offset + DA_BYTE_LEN);
        let currency = bytes[offset..offset + DA_BYTE_LEN].try_into()?;
        offset += DA_BYTE_LEN;

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

#[cfg(test)]
mod tests {
    use cesium_crypto::mldsa::da::DisplayAddress;

    use super::*;

    #[test]
    fn test_currency_amount_metadata() {
        let currency = *DisplayAddress::new().as_bytes();
        let amount = 1000;
        let metadata = CurrencyHolderData { currency, amount };

        let bytes = metadata.to_bytes();
        let metadata2 = CurrencyHolderData::try_from_bytes(&bytes).unwrap();

        assert_eq!(metadata.currency, metadata2.currency);
        assert_eq!(metadata.amount, metadata2.amount);
    }
}
