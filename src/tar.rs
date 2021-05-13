use std::{io::Write, path::{Path, PathBuf}};

use crate::{ChangeNotifier, FsChangeWatcher, FsEntry, FsProcessor, change_watcher::FsNode};

use anyhow::Result;
use std::collections::HashMap;
use tar_impl::{Builder, HeaderMode};

pub struct TarProcessor<W: Write>(ChangeNotifier<TarNotifier<W>>);

struct TarNotifier<W: Write> {
    builder: Builder<W>,
}

impl<W: Write> TarNotifier<W> {
    fn notify_file_or_sym_added(&mut self, path: &Path) -> Result<()> {
        let rel_path = path.strip_prefix("/")?;
        self.builder.append_path_with_name(path, rel_path)?;
        Ok(())
    }
    fn notify_folder_added(&mut self, path: &Path) -> Result<()> {
        let rel_path = path.strip_prefix("/")?;
        self.builder.append_dir(rel_path, path)?;
        Ok(())
    }
}

impl<W: Write> TarProcessor<W> {
    pub fn new(writer: W) -> Self {
        let mut builder = Builder::new(writer);
        builder.follow_symlinks(false);
        builder.mode(HeaderMode::Complete);
        let notifier = ChangeNotifier::new(TarNotifier{builder});
        Self(notifier)
    }
}

impl<W: Write> FsChangeWatcher for TarNotifier<W> {
    fn notify_file_added(&mut self, path: &Path) -> Result<()> {
        self.notify_file_or_sym_added(path)
    }

    fn notify_file_changed(&mut self, path: &Path) -> Result<()> {
        self.notify_file_or_sym_added(path)
    }

    fn notify_file_removed(&mut self, _path: &Path) -> Result<()> {
        // TODO add file removed marker
        Ok(())
    }

    fn notify_symlink_added(&mut self, path: &Path) -> Result<()> {
        self.notify_file_or_sym_added(path)
    }

    fn notify_symlink_changed(&mut self, path: &Path) -> Result<()> {
        self.notify_file_or_sym_added(path)
    }

    fn notify_symlink_removed(&mut self, _path: &Path) -> Result<()> {
        // TODO add file removed marker
        Ok(())
    }

    fn notify_folder_added(&mut self, path: &Path) -> Result<()> {
        self.notify_folder_added(path)
    }

    fn notify_folder_changed(&mut self, path: &Path) -> Result<()> {
        self.notify_folder_added(path)
    }

    fn notify_folder_removed(&mut self, _path: &Path) -> Result<()> {
        // TODO add file removed marker
        Ok(())
    }
}

impl<W: Write> FsProcessor for TarProcessor<W> {
    type Item = FsNode;

    fn process_file(&mut self, path: &Path, previous: Option<Self::Item>) -> Result<Self::Item> {
        self.0.process_file(path, previous)
    }

    fn process_symlink(&mut self, path: &Path, previous: Option<Self::Item>) -> Result<Self::Item> {
        self.0.process_symlink(path, previous)
    }

    fn process_folder(
        &mut self,
        path: &Path,
        sub: HashMap<PathBuf, FsEntry<Self::Item>>,
        previous: Option<Self::Item>,
    ) -> Result<Self::Item> {
        self.0.process_folder(path, sub, previous)
    }
}