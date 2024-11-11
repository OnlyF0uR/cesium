#[derive(Debug)]
pub enum StorageError {
    RocksDBError(rocksdb::Error),
    AsyncError(String),
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            StorageError::RocksDBError(err) => write!(f, "RocksDB error: {}", err),
            StorageError::AsyncError(err) => write!(f, "Async error: {}", err),
        }
    }
}

impl std::error::Error for StorageError {}
