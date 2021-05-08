use anyhow::Result;

mod lru;

/// notify a block has been found/generated
pub trait BlockVisitor {
    type Block;
    type BlockId;
    fn visit_block(&mut self, id: Self::BlockId, block: Self::Block) -> Result<()>;
}
