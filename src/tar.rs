use std::{io::Write, path::Path};

use crate::{FsEntry, FsProcessor};

use anyhow::Result;
use std::collections::HashMap;
use std::ffi::OsString;
use tar_impl::{Builder, HeaderMode};

pub struct TarProcessor<W: Write> {
    builder: Builder<W>,
}

impl<W: Write> TarProcessor<W> {
    pub fn new(writer: W) -> Self {
        let mut builder = Builder::new(writer);
        builder.follow_symlinks(false);
        builder.mode(HeaderMode::Complete);
        Self { builder }
    }
}

impl<W: Write> FsProcessor for TarProcessor<W> {
    type Item = Option<bool>;

    fn process_file(&mut self, path: &Path) -> Result<Self::Item> {
        let rel_path = path.strip_prefix("/")?;
        self.builder.append_path_with_name(path, rel_path)?;
        Ok(None)
    }

    fn process_symlink(&mut self, path: &Path) -> Result<Self::Item> {
        let rel_path = path.strip_prefix("/")?;
        self.builder.append_path_with_name(path, rel_path)?;
        Ok(None)
    }

    fn process_folder(
        &mut self,
        path: &Path,
        _sub: HashMap<OsString, FsEntry<Self::Item>>,
    ) -> Result<Self::Item> {
        let rel_path = path.strip_prefix("/")?;
        self.builder.append_dir(rel_path, path)?;
        Ok(None)
    }
}
