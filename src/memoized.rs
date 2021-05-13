use anyhow::Result;
use std::{
    cmp::max, collections::HashMap, fs::read_dir, marker::PhantomData, path::Path, time::SystemTime,
};

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
        Op: FnOnce(Option<I>) -> Result<I>;

    fn end_session(self) -> Result<Self::Cache>;

    fn get_id(&self) -> u32;
}

pub trait MemoizedFsCache<I> {
    type Session: MemoizedFsCacheSession<I, Cache = Self>;
    fn start_session(self) -> Result<Self::Session>;
}

/// Use a database and file mtime to skip visit of unchanged fs items
pub struct MemoizedFsWalker<I, C: MemoizedFsCache<I>> {
    cache: C,
    _ph: PhantomData<I>,
}

impl<I, C: MemoizedFsCache<I>> MemoizedFsWalker<I, C> {
    pub fn new(cache: C) -> Self {
        Self {
            cache,
            _ph: PhantomData,
        }
    }
    pub fn start_processing<F: FsProcessor<Item = I>>(
        self,
        fs_processor: F,
    ) -> Result<MemoizedFsWalkerSession<F, C::Session>> {
        Ok(MemoizedFsWalkerSession {
            fs_processor,
            session: self.cache.start_session()?,
        })
    }
}

/// Use a database and file mtime to skip visit of unchanged fs items
pub struct MemoizedFsWalkerSession<F: FsProcessor, S: MemoizedFsCacheSession<F::Item>> {
    fs_processor: F,
    session: S,
}

impl<F: FsProcessor, S: MemoizedFsCacheSession<F::Item>> MemoizedFsWalkerSession<F, S> {
    pub fn add_path(
        &mut self,
        path: impl AsRef<Path>,
        mount_path: impl AsRef<Path>,
    ) -> Result<FsEntry<F::Item>> {
        let path = path.as_ref();
        let mount_path = mount_path.as_ref();
        let meta = path.symlink_metadata()?;
        let mtime = meta.modified()?;
        let ft = meta.file_type();
        if ft.is_file() {
            let fs_hasher = &mut self.fs_processor;
            self.session
                .get_update_entry_from_cache(path, mtime, |opt_prev| {
                    fs_hasher.process_file(path, mount_path, opt_prev)
                })
        } else if ft.is_dir() {
            let mut max_mtime = mtime;
            let mut entry_map = HashMap::new();
            for sub in read_dir(path)? {
                let entry = sub?;
                let new_mount_path = mount_path.join(entry.file_name());
                let folder_entry = self.add_path(&entry.path(), new_mount_path)?;
                max_mtime = max(max_mtime, folder_entry.mtime);
                entry_map.insert(entry.file_name().into(), folder_entry);
            }

            let fs_hasher = &mut self.fs_processor;
            self.session
                .get_update_entry_from_cache(path, mtime, |opt_prev| {
                    fs_hasher.process_folder(path, mount_path, entry_map, opt_prev)
                })
        } else {
            let fs_hasher = &mut self.fs_processor;
            self.session
                .get_update_entry_from_cache(path, mtime, |opt_prev| {
                    fs_hasher.process_symlink(path, mount_path, opt_prev)
                })
        }
    }

    pub fn finish_processing(self) -> Result<(MemoizedFsWalker<F::Item, S::Cache>, u32)> {
        let session_id = self.session.get_id();
        Ok((
            MemoizedFsWalker::new(self.session.end_session()?),
            session_id,
        ))
    }
}
