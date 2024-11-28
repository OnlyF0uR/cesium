use cesium_crypto::mldsa::da::{DABytes, DA_BYTE_LEN};

pub struct NFTHolderData {
    name_len: u32,
    name: String,
    url_len: u32,
    uri: String,
    creator_count: u32,
    creators: Vec<DABytes>,
}

macro_rules! bounds_check {
    ($bytes:expr, $DA_BYTE_LEN:expr) => {
        if $bytes.len() < $DA_BYTE_LEN {
            // TODO: Return an error instead of panicking
            panic!("Out of bounds NFT metadata bytes");
        }
    };
}

impl NFTHolderData {
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // Each field is prefixed with a usize length
        let mut offset = 0;
        bounds_check!(bytes, offset + 4);
        let name_len = u32::from_le_bytes(bytes[offset..offset + 4].try_into()?);
        offset += 4;

        bounds_check!(bytes, offset + name_len as usize);
        let name = String::from_utf8(bytes[offset..(offset + name_len as usize)].to_vec())?;
        offset += name_len as usize;

        bounds_check!(bytes, offset + 4);
        let url_len = u32::from_le_bytes(bytes[offset..offset + 4].try_into()?);
        offset += 4;

        bounds_check!(bytes, offset + url_len as usize);
        let uri = String::from_utf8(bytes[offset..(offset + url_len as usize)].to_vec())?;
        offset += url_len as usize;

        bounds_check!(bytes, offset + 4);
        let creator_count = u32::from_le_bytes(bytes[offset..offset + 4].try_into()?);
        offset += 4;

        bounds_check!(bytes, offset + DA_BYTE_LEN * creator_count as usize);
        let mut creators = Vec::new();
        for _ in 0..creator_count {
            let pk: [u8; DA_BYTE_LEN] = bytes[offset..offset + DA_BYTE_LEN].try_into().unwrap();
            offset = offset + DA_BYTE_LEN;
            creators.push(pk);
        }

        Ok(Self {
            name_len,
            name,
            url_len,
            uri,
            creator_count,
            creators,
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.name_len.to_le_bytes());
        bytes.extend_from_slice(self.name.as_bytes());
        bytes.extend_from_slice(&self.url_len.to_le_bytes());
        bytes.extend_from_slice(self.uri.as_bytes());
        bytes.extend_from_slice(&self.creator_count.to_le_bytes());
        for creator in &self.creators {
            bytes.extend_from_slice(creator);
        }
        bytes
    }
}

#[cfg(test)]
mod tests {
    use cesium_crypto::mldsa::da::DisplayAddress;

    use super::*;

    #[test]
    fn test_nft_metadata() {
        let name = "Test NFT".to_string();
        let uri = "https://127.0.0.1".to_string();
        let creator_count = 2;
        let creators = vec![
            *DisplayAddress::new().as_bytes(),
            *DisplayAddress::new().as_bytes(),
        ];
        let metadata = NFTHolderData {
            name_len: name.len() as u32,
            name,
            url_len: uri.len() as u32,
            uri,
            creator_count,
            creators,
        };

        let bytes = metadata.to_bytes();
        let metadata2 = NFTHolderData::try_from_bytes(&bytes).unwrap();

        assert_eq!(metadata.name, metadata2.name);
        assert_eq!(metadata.uri, metadata2.uri);
        assert_eq!(metadata.creator_count, metadata2.creator_count);
        assert_eq!(metadata.creators, metadata2.creators);
    }
}
