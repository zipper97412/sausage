use std::{
    io::Write,
    path::{Path, PathBuf},
    time::SystemTime,
};

use crate::{change_watcher::FsNode, ChangeNotifier, FsChangeWatcher, FsEntry, FsProcessor};

use anyhow::Result;
use std::collections::HashMap;
use tar_impl::{Builder, Header, HeaderMode};

pub struct TarProcessor<W: Write>(ChangeNotifier<TarNotifier<W>>);

struct TarNotifier<W: Write> {
    builder: Builder<W>,
}

impl<W: Write> TarProcessor<W> {
    pub fn new(writer: W) -> Self {
        let mut builder = Builder::new(writer);
        builder.follow_symlinks(false);
        builder.mode(HeaderMode::Complete);
        let notifier = ChangeNotifier::new(TarNotifier { builder });
        Self(notifier)
    }
}

impl<W: Write> FsChangeWatcher for TarNotifier<W> {
    fn notify_file_added(&mut self, path: &Path, mount_path: &Path) -> Result<()> {
        self.builder.append_path_with_name(path, mount_path)?;
        Ok(())
    }

    fn notify_file_changed(&mut self, path: &Path, mount_path: &Path) -> Result<()> {
        self.builder.append_path_with_name(path, mount_path)?;
        Ok(())
    }

    fn notify_file_removed(&mut self, _path: &Path, mount_path: &Path) -> Result<()> {
        let mut header = Header::new_gnu();
        header.set_size(0);
        header.set_mtime(
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)?
                .as_secs(),
        );
        let data: &[u8] = &[];
        let new_filename = format!(
            "{}.DELETED",
            mount_path.file_name().unwrap().to_string_lossy()
        );
        self.builder
            .append_data(&mut header, mount_path.with_file_name(new_filename), data)?;
        Ok(())
    }

    fn notify_symlink_added(&mut self, path: &Path, mount_path: &Path) -> Result<()> {
        self.builder.append_path_with_name(path, mount_path)?;
        Ok(())
    }

    fn notify_symlink_changed(&mut self, path: &Path, mount_path: &Path) -> Result<()> {
        self.builder.append_path_with_name(path, mount_path)?;
        Ok(())
    }

    fn notify_symlink_removed(&mut self, _path: &Path, mount_path: &Path) -> Result<()> {
        let mut header = Header::new_gnu();
        header.set_size(0);
        header.set_mtime(
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)?
                .as_secs(),
        );
        let data: &[u8] = &[];
        let new_filename = format!(
            "{}.DELETED",
            mount_path.file_name().unwrap().to_string_lossy()
        );
        self.builder
            .append_data(&mut header, mount_path.with_file_name(new_filename), data)?;
        Ok(())
    }

    fn notify_folder_added(&mut self, path: &Path, mount_path: &Path) -> Result<()> {
        self.builder.append_dir(mount_path, path)?;
        Ok(())
    }

    fn notify_folder_changed(&mut self, path: &Path, mount_path: &Path) -> Result<()> {
        self.builder.append_dir(mount_path, path)?;
        Ok(())
    }

    fn notify_folder_removed(&mut self, _path: &Path, mount_path: &Path) -> Result<()> {
        let mut header = Header::new_gnu();
        header.set_size(0);
        header.set_mtime(
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)?
                .as_secs(),
        );
        let data: &[u8] = &[];
        let new_filename = format!(
            "{}.DELETED",
            mount_path.file_name().unwrap().to_string_lossy()
        );
        self.builder
            .append_data(&mut header, mount_path.with_file_name(new_filename), data)?;
        Ok(())
    }
}

impl<W: Write> FsProcessor for TarProcessor<W> {
    type Item = FsNode;

    fn process_file(
        &mut self,
        path: &Path,
        mount_path: &Path,
        previous: Option<Self::Item>,
    ) -> Result<Self::Item> {
        self.0.process_file(path, mount_path, previous)
    }

    fn process_symlink(
        &mut self,
        path: &Path,
        mount_path: &Path,
        previous: Option<Self::Item>,
    ) -> Result<Self::Item> {
        self.0.process_symlink(path, mount_path, previous)
    }

    fn process_folder(
        &mut self,
        path: &Path,
        mount_path: &Path,
        sub: HashMap<PathBuf, FsEntry<Self::Item>>,
        previous: Option<Self::Item>,
    ) -> Result<Self::Item> {
        self.0.process_folder(path, mount_path, sub, previous)
    }
}
