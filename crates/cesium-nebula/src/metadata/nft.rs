use cesium_crypto::keys::{PublicKeyBytes, PUB_BYTE_LEN};

pub struct NFTMetadata {
    name_len: u32,
    name: String,
    url_len: u32,
    uri: String,
    creator_count: u32,
    creators: Vec<PublicKeyBytes>,
}

macro_rules! bounds_check {
    ($bytes:expr, $pub_byte_len:expr) => {
        if $bytes.len() < $pub_byte_len {
            // TODO: Return an error instead of panicking
            panic!("Out of bounds NFT metadata bytes");
        }
    };
}

impl NFTMetadata {
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

        bounds_check!(bytes, offset + PUB_BYTE_LEN * creator_count as usize);
        let mut creators = Vec::new();
        for _ in 0..creator_count {
            let pk: [u8; PUB_BYTE_LEN] = bytes[offset..offset + PUB_BYTE_LEN].try_into().unwrap();
            offset = offset + PUB_BYTE_LEN;
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
