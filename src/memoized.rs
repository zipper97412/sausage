use anyhow::Result;
use std::{cmp::max, collections::HashMap, fs::read_dir, path::Path, time::SystemTime};

use super::{FsEntry, FsProcessor};

pub trait MemoizedFsCacheSession<I> {
    type Cache: MemoizedFsCache<I, Session = Self>;
    // get the latest id for this path and mtime, optionally computing the new id if necessary
    fn get_update_entry_from_cache<Op>(
        &mut self,
        path: &Path,
        mtime: SystemTime,
        compute_item: Op,
    ) -> Result<FsEntry<I>>
    where
        Op: FnOnce() -> Result<I>;

    fn end_session(self) -> Result<Self::Cache>;
}

pub trait MemoizedFsCache<I> {
    type Session: MemoizedFsCacheSession<I, Cache = Self>;
    fn start_session(self) -> Result<Self::Session>;
}

/// Use a database and file mtime to skip visit of unchanged fs items
pub struct MemoizedFsWalker<F: FsProcessor> {
    fs_processor: F,
}

impl<F: FsProcessor> MemoizedFsWalker<F> {
    pub fn new(fs_processor: F) -> Self {
        MemoizedFsWalker { fs_processor }
    }

    pub fn hash_path<C: MemoizedFsCache<F::Item>>(
        &mut self,
        cache: C,
        path: impl AsRef<Path>,
    ) -> Result<(FsEntry<F::Item>, C)> {
        let path = path.as_ref().canonicalize()?;
        let ft = path.symlink_metadata()?.file_type();
        let mut session = cache.start_session()?;
        let entry = if ft.is_file() {
            self.hash_file(&mut session, path)?
        } else if ft.is_dir() {
            self.hash_folder(&mut session, path)?
        } else {
            self.hash_symlink(&mut session, path)?
        };

        Ok((entry, session.end_session()?))
    }

    fn hash_file<S: MemoizedFsCacheSession<F::Item>>(
        &mut self,
        session: &mut S,
        path: impl AsRef<Path>,
    ) -> Result<FsEntry<F::Item>> {
        let path = path.as_ref();
        let mtime = path.symlink_metadata()?.modified()?;
        let fs_hasher = &mut self.fs_processor;
        session.get_update_entry_from_cache(path, mtime, || fs_hasher.process_file(path))
    }

    fn hash_folder<S: MemoizedFsCacheSession<F::Item>>(
        &mut self,
        session: &mut S,
        path: impl AsRef<Path>,
    ) -> Result<FsEntry<F::Item>> {
        let path = path.as_ref();
        let meta = path.symlink_metadata()?;
        let mtime = meta.modified()?;
        let ft = meta.file_type();
        if ft.is_file() {
            let fs_hasher = &mut self.fs_processor;
            session.get_update_entry_from_cache(path, mtime, || fs_hasher.process_file(path))
        } else if ft.is_dir() {
            let mut max_mtime = mtime;
            let mut entry_map = HashMap::new();
            for sub in read_dir(path)? {
                let entry = sub?;
                let folder_entry = self.hash_folder(session, &entry.path())?;
                max_mtime = max(max_mtime, folder_entry.mtime);
                entry_map.insert(entry.file_name(), folder_entry);
            }

            let fs_hasher = &mut self.fs_processor;
            session.get_update_entry_from_cache(path, mtime, || {
                fs_hasher.process_folder(path, entry_map)
            })
        } else {
            let fs_hasher = &mut self.fs_processor;
            session.get_update_entry_from_cache(path, mtime, || fs_hasher.process_symlink(path))
        }
    }

    fn hash_symlink<S: MemoizedFsCacheSession<F::Item>>(
        &mut self,
        session: &mut S,
        path: impl AsRef<Path>,
    ) -> Result<FsEntry<F::Item>> {
        let path = path.as_ref();
        let mtime = path.symlink_metadata()?.modified()?;
        let fs_hasher = &mut self.fs_processor;
        session.get_update_entry_from_cache(path, mtime, || fs_hasher.process_symlink(path))
    }
}
