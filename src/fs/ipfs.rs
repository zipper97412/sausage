
use std::{fs::{File, Metadata}, io::{Read, Write}, path::Path};

use anyhow::Result;
use cid::Cid;

use crate::{block::BlockVisitor, fs::FsVisitor};

/// implement Unix File system block generation for each items (IPFS unixfs v1 format)
struct IpfsUnixFsVisitor<B> {
    block_visitor: B,
}

impl<B> FsVisitor for IpfsUnixFsVisitor<B>
where
    B: BlockVisitor<BlockId = Cid, Block = Vec<u8>>,
{
    type FsId = Cid;
    fn visit_file(&mut self, path: &Path) -> Result<Self::FsId> {
        //split the file in blocks using rabin and FileAdder
        //send each blocks using block_visitor
        //return the root cid of the file
        todo!()
    }

    fn visit_folder(&mut self, path: &Path) -> Result<Self::FsId> {
        //create a BufferingTreeBuilder and recursively traverse the folder and add everything 
        //convert the BufferingTreeBuilder to PostOrderIterator and send every blocks using block_visitor
        //return the root cid of the folder

        // this implementation will not be used by UnixFsVisitorCache, so implementation is optional
        todo!()
    }

    fn visit_symlink(&mut self, path: &Path) -> Result<Self::FsId> {
        //create a block for the symlink, send it and return the cid
        todo!()
    }
}

impl<B> BlockVisitor for IpfsUnixFsVisitor<B> 
where
    B: BlockVisitor<BlockId = Cid, Block = Vec<u8>>,
{
    type BlockId = Cid;
    type Block = Vec<u8>;
    fn visit_block(&mut self, id: Self::BlockId, block: Self::Block) -> Result<()> {
        self.block_visitor.visit_block(id, block)
    }
}

/// same as UnixFsVisitor but use a cache database and file mtime to skip block generation of unchanged fs items
/// blocks that are not generated should already be sent from previous runs
struct IpfsUnixFsVisitorCache<F> {
    inner: F
}

impl<F: FsVisitor + BlockVisitor> FsVisitor for IpfsUnixFsVisitorCache<F> 
where
    F: BlockVisitor<BlockId = Cid, Block = Vec<u8>>,
    F: FsVisitor<FsId = Cid>,
{
    type FsId = Cid;
    fn visit_file(&mut self, path: &Path) -> Result<Self::FsId> {
        //compare fs_mtime and db_mtime for this file path
        //if fs_mtimt > db_mtime, call inner.visit_file() and update db
        //else use Self::Id from DB
        todo!()
    }

    fn visit_folder(&mut self, path: &Path) -> Result<Self::FsId> {
        //recursively find each changed paths in the folder and flag changed path in db
        //create a BufferingTreeBuilder and recursively traverse the folder and add everything
        //  - for folders, first check in db if they have changed before going down in recursion
        //  - for files/symlinks, first check in db if they have changed before calling inner.visit_*
        //convert the BufferingTreeBuilder to PostOrderIterator and send every blocks using inner.block_visitor
        //also update database mtime for each tree nodes for next run
        todo!()
    }

    fn visit_symlink(&mut self, path: &Path) -> Result<Self::FsId> {
        //compare fs_mtime and db_mtime for this file path
        //if fs_mtimt > db_mtime, call inner.visit_file() and update db
        //else use Self::Id from DB
        todo!()
    }
}

impl<F> IpfsUnixFsVisitorCache<F>
where
    F: BlockVisitor<BlockId = Cid, Block = Vec<u8>>,
    F: FsVisitor<FsId = Cid>,
{
    /// visit a given path, dispathing to FsVisitor's methods if necessary
    fn visit_path(&mut self, path: &Path) -> Result<Cid> {
        todo!()
    }
}


/// serialize and write blocks to a writer using the same TLV as the CAR format for blocks.
/// there is no header, but the last block should be the root of the synced path so a CAR can be generated from this later
struct CarBlockWriter<W> {
    writer: W,
}

impl<W> BlockVisitor for CarBlockWriter<W> 
where 
    W: Write,
{
    type Block = Vec<u8>;
    type BlockId = Cid;
    fn visit_block(&mut self, id: Self::BlockId, block: Self::Block) -> Result<()> {
        //serialize and write the block to the writer using car's TLV format
        todo!()
    }
}