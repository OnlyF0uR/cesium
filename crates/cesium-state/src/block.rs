use cesium_material::{block::Block, keys::KeyPair};
use std::sync::Arc;
use tokio::sync::OnceCell;

pub struct BlockCache {
    pub block: Block,
}

impl BlockCache {
    async fn initialize() -> Result<BlockCache, Box<dyn std::error::Error + Send + Sync>> {
        // TODO: Obtain latest block from disk
        let block = Block::new(0, &KeyPair::create(), Vec::new())?;
        Ok(BlockCache { block })
    }
}

static BC_INSTANCE: OnceCell<Arc<BlockCache>> = OnceCell::const_new();

pub async fn get_block_cache() -> Arc<BlockCache> {
    BC_INSTANCE
        .get_or_init(|| async {
            let bc = BlockCache::initialize()
                .await
                .expect("Failed to initialize block cache");
            Arc::new(bc)
        })
        .await
        .clone()
}
