use once_cell::sync::Lazy;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StandardToken {
    Cesium,
    WBTC,
    WETH,
    MER,
}

pub struct TokenMetadata {
    address: &'static str,
    decimals: u8,
    short_name: &'static str,
    full_name: &'static str,
}

pub const NATIVE_TOKEN: &str = "cesium111111111111111111111111111111111111111111111111111111111111";
// [99, 101, 115, 105, 117, 109, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49, 49]
pub const NATIVE_DECIMALS: u8 = 12;

pub const MIN_DECIMALS: u8 = 8;
pub const MAX_DECIMALS: u8 = 24;

// Static mapping of token metadata
static TOKEN_METADATA: Lazy<HashMap<StandardToken, TokenMetadata>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert(
        StandardToken::Cesium,
        TokenMetadata {
            address: NATIVE_TOKEN,
            decimals: NATIVE_DECIMALS,
            short_name: "cesium",
            full_name: "Cesium",
        },
    );
    m.insert(
        StandardToken::WBTC,
        TokenMetadata {
            address: "wbtc11111111111111111111111111111111111111111111111111111111111111",
            decimals: 8,
            short_name: "wbtc",
            full_name: "Wrapped Bitcoin",
        },
    );
    m.insert(
        StandardToken::WETH,
        TokenMetadata {
            address: "weth11111111111111111111111111111111111111111111111111111111111111",
            decimals: 18,
            short_name: "weth",
            full_name: "Wrapped Ether",
        },
    );
    // MER token for the Merodex exchange
    m.insert(
        StandardToken::MER,
        TokenMetadata {
            address: "mer111111111111111111111111111111111111111111111111111111111111111",
            decimals: 18,
            short_name: "mer",
            full_name: "Mero",
        },
    );
    m
});

// Reverse lookups using once_cell
static ADDRESS_TO_TOKEN: Lazy<HashMap<&'static str, StandardToken>> = Lazy::new(|| {
    TOKEN_METADATA
        .iter()
        .map(|(token, metadata)| (metadata.address, *token))
        .collect()
});

static SHORT_NAME_TO_TOKEN: Lazy<HashMap<&'static str, StandardToken>> = Lazy::new(|| {
    TOKEN_METADATA
        .iter()
        .map(|(token, metadata)| (metadata.short_name, *token))
        .collect()
});

impl StandardToken {
    pub fn iter() -> impl Iterator<Item = StandardToken> {
        [
            StandardToken::Cesium,
            StandardToken::WBTC,
            StandardToken::WETH,
            StandardToken::MER,
        ]
        .iter()
        .copied()
    }
    // pub fn try_from_address(address: &str) -> Result<Self, &'static str> {
    //     if let Some(token) = StandardToken::from_address(address) {
    //         Ok(token)
    //     } else {
    //         Err("Invalid token address")
    //     }
    // }

    pub fn is_standard_token(address: &str) -> bool {
        // let address = address.to_lowercase();
        StandardToken::from_address(&address).is_some()
    }

    pub fn from_address(address: &str) -> Option<Self> {
        ADDRESS_TO_TOKEN.get(address).copied()
    }

    pub fn from_short_name(name: &str) -> Option<Self> {
        SHORT_NAME_TO_TOKEN.get(name).copied()
    }

    pub fn metadata(&self) -> &TokenMetadata {
        // Safe to unwrap as we know all enum variants are in the map
        TOKEN_METADATA.get(self).unwrap()
    }

    pub fn address(&self) -> &'static str {
        self.metadata().address
    }

    pub fn decimals(&self) -> u8 {
        self.metadata().decimals
    }

    pub fn short_name(&self) -> &'static str {
        self.metadata().short_name
    }

    pub fn full_name(&self) -> &'static str {
        self.metadata().full_name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_token() {
        // loop over the standard tokens
        for token in StandardToken::iter() {
            // First check the length of the addresses
            assert_eq!(token.address().len(), 66);
            // Now lets check the decimals
            assert!(token.decimals() >= MIN_DECIMALS);
            assert!(token.decimals() <= MAX_DECIMALS);
            // Max length of short name
            assert!(token.short_name().len() >= 3);
            assert!(token.short_name().len() <= 7);
            // Long name restrictions
            assert!(token.full_name().len() >= 3);
            assert!(token.full_name().len() <= 21);
            // Now lets check the reverse lookups
            assert_eq!(StandardToken::from_address(token.address()), Some(token));
            assert_eq!(
                StandardToken::from_short_name(token.short_name()),
                Some(token)
            );
        }
    }

    #[test]
    fn test_standard_token_invalid_lookups() {
        // Test invalid lookups
        assert_eq!(StandardToken::from_address("invalid"), None);
        assert_eq!(StandardToken::from_short_name("invalid"), None);
    }
}
