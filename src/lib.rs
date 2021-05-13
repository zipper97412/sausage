mod memoized;
pub use memoized::{MemoizedFsCache, MemoizedFsCacheSession, MemoizedFsWalker};

#[cfg(feature = "sqlite")]
mod sqlite;
#[cfg(feature = "sqlite")]
pub use sqlite::{setup_sqlite_cache, rollback_before_session_id};

//mod ipfs;

#[cfg(feature = "tar")]
mod tar;
#[cfg(feature = "tar")]
pub use tar::TarProcessor;

#[cfg(feature = "change_watcher")]
mod change_watcher;
#[cfg(feature = "change_watcher")]
pub use change_watcher::{ChangeNotifier, FsChangeWatcher};

use std::{collections::HashMap, path::{Path, PathBuf}, time::SystemTime};

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
    fn process_file(&mut self, path: &Path, previous: Option<Self::Item>) -> Result<Self::Item>;

    /// process a symlink, return an item, this item will be cached in the `MemoizedFsWalker` database
    fn process_symlink(&mut self, path: &Path, previous: Option<Self::Item>) -> Result<Self::Item>;

    /// process a symlink, return an item, this item will be cached in the `MemoizedFsWalker` database
    fn process_folder(
        &mut self,
        path: &Path,
        sub: HashMap<PathBuf, FsEntry<Self::Item>>,
        previous: Option<Self::Item>,
    ) -> Result<Self::Item>;
}
