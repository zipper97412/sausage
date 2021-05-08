use anyhow::Result;

use super::BlockVisitor;

/// filter out already seen blocks using the BlockId, a DB file is used as LRU Cache to remember BlockIds
struct BlockLRUCacheFilter<T> {
    inner: T,
    lru: (),
}

impl<T: BlockVisitor> BlockVisitor for BlockLRUCacheFilter<T> {
    type Block = T::Block;
    type BlockId = T::BlockId;
    fn visit_block(&mut self, id: Self::BlockId, block: Self::Block) -> Result<()> {
        //check if block id is in lru
        // HIT: update cache and return
        // MISS: update cache and visit block with inner
        todo!()
    }
}
