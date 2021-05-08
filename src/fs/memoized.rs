use std::{cmp::max, fs::read_dir, path::{Path}, time::SystemTime};
use anyhow::{Result, anyhow};

use super::{FsProcessor, FolderTreeBuilder};



/// Use a database and file mtime to skip visit of unchanged fs items
pub struct MemoizedFsWalker<F> {
    fs_processor: F,
    db: (),
}

impl<F: FsProcessor> MemoizedFsWalker<F> {

    // get the latest id for this path and mtime, optionally computing the new id if necessary
    fn get_update_id_from_cache<Op>(db: &mut (), path: &Path, mtime: SystemTime, compute_id: Op) -> Result<F::Item> 
    where 
        Op: FnOnce() -> Result<F::Item>,
    {
        todo!()
    }

    fn get_id_from_cache(db: &mut (), path: &Path, mtime: SystemTime) -> Result<(Option<F::Item>, bool)> {
        todo!()
    }


    fn build_folder_rec(&mut self, builder: &mut F::FolderBuilder, path: &Path) -> Result<SystemTime> {
        let meta = path.symlink_metadata()?;
        let c_mtime = meta.modified()?;
        let ft = meta.file_type();
        if ft.is_file() {
            let fs_hasher = &mut self.fs_processor;
            let id = Self::get_update_id_from_cache(&mut self.db, path, c_mtime, || {
                fs_hasher.process_file(path)
            })?;
            builder.visit_file(path, id, c_mtime)?;
            Ok(c_mtime)
        } else if ft.is_dir() {
            let mut sub_mtime = SystemTime::UNIX_EPOCH;
            for sub in read_dir(path)? {
                let entry = sub?;
                sub_mtime = max(sub_mtime, self.build_folder_rec(builder, &entry.path())?);
            }
            if sub_mtime > c_mtime {
                Ok(sub_mtime)
            } else {
                let final_mtime = max(c_mtime, sub_mtime);
                let (opt_id, _) = Self::get_id_from_cache(&mut self.db, path, c_mtime)?;
                if let Some(id) = opt_id {
                    builder.visit_folder(path, id, final_mtime)?;
                }
                Ok(final_mtime)
            }
        } else {
            let fs_hasher = &mut self.fs_processor;
            let id = Self::get_update_id_from_cache(&mut self.db, path, c_mtime, || {
                fs_hasher.process_symlink(path)
            })?;
            builder.visit_symlink(path, id, c_mtime)?;
            Ok(c_mtime)
        }
    }

    pub fn hash_path(&mut self, path: &Path) -> Result<F::Item> {
        let ft = path.symlink_metadata()?.file_type();
        if ft.is_file() {
            self.hash_file(path)
        } else if ft.is_dir() {
            self.hash_folder(path)
        } else {
            self.hash_symlink(path)
        }
    }

    fn hash_file(&mut self, path: &Path) -> Result<F::Item> {
        let c_mtime = path.symlink_metadata()?.modified()?;
        let fs_hasher = &mut self.fs_processor;
        Self::get_update_id_from_cache(&mut self.db, path, c_mtime, || {
            fs_hasher.process_file(path)
        })
    }

    fn hash_folder(&mut self, path: &Path) -> Result<F::Item> {
        let mut builder = self.fs_processor.folder_tree_builder(path);
        
        self.build_folder_rec(&mut builder, path)?;
        
        // consume the tree builder and update ids
        let mut last_id = None;
        for (path, c_mtime, id) in builder.into_iter() {
            let id = Self::get_update_id_from_cache(&mut self.db, &path, c_mtime, || {
                Ok(id)
            })?;
            last_id = Some(id);
        }
        last_id.ok_or_else(|| anyhow!("there is no id in tree builder"))
    }

    fn hash_symlink(&mut self, path: &Path) -> Result<F::Item> {
        let c_mtime = path.symlink_metadata()?.modified()?;
        let fs_hasher = &mut self.fs_processor;
        Self::get_update_id_from_cache(&mut self.db, path, c_mtime, || {
            fs_hasher.process_symlink(path)
        })
    }
}

