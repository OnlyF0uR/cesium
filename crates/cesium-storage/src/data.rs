use serde::{Deserialize, Serialize};

use cesium_material::serializer::Array48;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataObject {
    #[serde(with = "Array48")]
    pub type_id: [u8; 48], // most commonly a token id in byte form
    pub data: Vec<u8>, // Use a slice for immutable data
}

impl DataObject {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(48 + self.data.len());
        bytes.extend(&self.type_id);
        bytes.extend(&self.data);
        bytes
    }

    pub fn from_bytes(
        bytes: &[u8],
    ) -> Result<DataObject, Box<dyn std::error::Error + Send + Sync>> {
        if bytes.len() < 48 {
            return Err("DataObject is empty".into());
        }

        let mut type_id = [0; 48];
        type_id.copy_from_slice(&bytes[0..48]);

        let data = bytes[48..].to_vec();
        Ok(DataObject { type_id, data })
    }
}
