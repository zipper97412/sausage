
use std::{path::{Path, PathBuf}, time::SystemTime};

use anyhow::Result;

mod memoized;
pub use memoized::MemoizedFsWalker;

//mod ipfs;

/// Folder builder interface, used by `FsProcessor`
pub trait FolderTreeBuilder {
    /// items to be buffered into this tree
    type Item;

    /// this iterator should return every folder item generated, at least one, and the last should be the root of the tree
    type FolderIdIter: Iterator<Item = (PathBuf, SystemTime, Self::Item)>;

    /// add a file item to the tree at a specific path, implicitly create parent folders
    fn visit_file(&mut self, path: &Path, item: Self::Item, mtime: SystemTime) -> Result<()>;

    /// add a folder item to the tree at a specific path, implicitly create parent folders
    fn visit_folder(&mut self, path: &Path, item: Self::Item, mtime: SystemTime) -> Result<()>;

    /// add a symlink item to the tree at a specific path, implicitly create parent folders
    fn visit_symlink(&mut self, path: &Path, item: Self::Item, mtime: SystemTime) -> Result<()>;

    /// finish the tree and turn this into an iterator that return folder tree nodes, at least one, and the last should be the root of the tree
    fn into_iter(self) -> Self::FolderIdIter;
}


/// File system processor implementation interface
pub trait FsProcessor {
    /// items yelded after processing a file system entry can be a hash, nothing or something hard to process
    
    type Item;

    /// processing a folder is done with a visitor/iterator patern
    type FolderBuilder: FolderTreeBuilder<Item = Self::Item>;

    /// process a file, return an item, this item will be cached in the `MemoizedFsWalker` database
    fn process_file(&mut self, path: &Path) -> Result<Self::Item>;

    /// process a symlink, return an item, this item will be cached in the `MemoizedFsWalker` database
    fn process_symlink(&mut self, path: &Path) -> Result<Self::Item>;

    /// return a folder builder implementation ready to be loaded with items
    fn folder_tree_builder(&mut self, path: &Path) -> Self::FolderBuilder;
}