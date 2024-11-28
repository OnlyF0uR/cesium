use errors::StorageError;
use once_cell::sync::Lazy;
use rocksdb::{Options, DB};
use std::sync::Arc;

pub struct RocksDBStore {
    db: Arc<DB>, // Arc for thread-safe shared access to DB
}

const DB_PATH: &str = ".cesiumdb";

impl RocksDBStore {
    pub fn instance() -> &'static RocksDBStore {
        static INSTANCE: Lazy<RocksDBStore> = Lazy::new(|| RocksDBStore::new(DB_PATH).unwrap());
        &INSTANCE
    }

    /// Creates a new instance of `RocksDBStore` with optimized settings for performance.
    /// Takes the path to the database as input.
    fn new(db_path: &str) -> Result<Self, StorageError> {
        let mut options = Options::default();

        // General options
        options.create_if_missing(true);

        // Performance optimizations
        options.set_max_background_jobs(4); // Compaction threads
        options.set_write_buffer_size(64 * 1024 * 1024); // Write buffer size (64MB)
        options.set_max_write_buffer_number(3); // Number of write buffers
        options.set_target_file_size_base(64 * 1024 * 1024); // Target file size for SST files
        options.set_level_compaction_dynamic_level_bytes(true);
        options.increase_parallelism(2);

        // resolve path based on cwd
        let db_path = std::env::current_dir().unwrap().join(db_path);
        let db_path = db_path.to_str().unwrap();

        // Initialize RocksDB
        let db = DB::open(&options, db_path).map_err(|e| StorageError::RocksDBError(e))?;
        Ok(RocksDBStore { db: Arc::new(db) })
    }

    /// Stores a key-value pair in the database.
    pub fn put(&self, key: &[u8], value: &[u8]) -> Result<(), StorageError> {
        self.db
            .put(key, value)
            .map_err(|e| StorageError::RocksDBError(e))
    }

    /// Retrieves a value for the given key from the database.
    pub fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError> {
        self.db.get(key).map_err(|e| StorageError::RocksDBError(e))
    }

    /// Asynchronously stores a key-value pair in the database.
    pub async fn async_put(&self, key: Vec<u8>, value: Vec<u8>) -> Result<(), StorageError> {
        let db = Arc::clone(&self.db);
        tokio::task::spawn_blocking(move || db.put(key, value))
            .await
            .map_err(|e| StorageError::AsyncError(e.to_string()))?
            .map_err(|e| StorageError::RocksDBError(e))
    }

    /// Asynchronously retrieves a value for the given key from the database.
    pub async fn async_get(&self, key: Vec<u8>) -> Result<Option<Vec<u8>>, StorageError> {
        let db = Arc::clone(&self.db);
        tokio::task::spawn_blocking(move || db.get(key))
            .await
            .map_err(|e| StorageError::AsyncError(e.to_string()))?
            .map_err(|e| StorageError::RocksDBError(e))
    }
}

pub mod errors;

#[cfg(test)]
mod tests {
    use cesium_crypto::mldsa::keypair::{SignerPair, ViewOperations};

    use super::*;

    #[test]
    fn test_storage_put() {
        let store = RocksDBStore::instance();
        let account = SignerPair::create();

        let da = account.get_da();

        let key = da.as_bytes();
        let value = "hello world".as_bytes();

        store.put(key, value).unwrap();

        let result = store.get(key).unwrap();
        assert_eq!(result.unwrap(), value);
    }

    #[tokio::test]
    async fn test_storage_put_async() {
        let store = RocksDBStore::instance();
        let account = SignerPair::create();

        let da = account.get_da();

        let key = da.as_bytes();
        let value = "hello world".as_bytes();

        store.async_put(key.to_vec(), value.to_vec()).await.unwrap();

        let result = store.async_get(key.to_vec()).await.unwrap();
        assert_eq!(result.unwrap(), value);
    }
}
