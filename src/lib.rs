mod memoized;
pub use memoized::{MemoizedFsCache, MemoizedFsCacheSession, MemoizedFsWalker};

mod sqlite;
pub use sqlite::setup_sqlite_cache;

//mod ipfs;

mod tar;
pub use tar::TarProcessor;

use std::{collections::HashMap, ffi::OsString, path::Path, time::SystemTime};

use anyhow::Result;

pub struct FsEntry<T> {
    pub item: T,
    pub mtime: SystemTime,
}

/// File system processor implementation interface
pub trait FsProcessor {
    /// items yelded after processing a file system entry can be a hash, nothing or something hard to process

    type Item;

    /// process a file, return an item, this item will be cached in the `MemoizedFsWalker` database
    fn process_file(&mut self, path: &Path) -> Result<Self::Item>;

    /// process a symlink, return an item, this item will be cached in the `MemoizedFsWalker` database
    fn process_symlink(&mut self, path: &Path) -> Result<Self::Item>;

    /// process a symlink, return an item, this item will be cached in the `MemoizedFsWalker` database
    fn process_folder(
        &mut self,
        path: &Path,
        sub: HashMap<OsString, FsEntry<Self::Item>>,
    ) -> Result<Self::Item>;
}
