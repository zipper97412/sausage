use std::{collections::{HashMap, HashSet}, path::{Path, PathBuf}};

use anyhow::Result;

use serde::{Serialize, Deserialize};

use crate::{FsEntry, FsProcessor};

pub trait FsChangeWatcher {
    fn notify_file_added(&mut self, path: &Path, mount_path: &Path) -> Result<()>;
    fn notify_file_changed(&mut self, path: &Path, mount_path: &Path) -> Result<()>;
    fn notify_file_removed(&mut self, path: &Path, mount_path: &Path) -> Result<()>;

    fn notify_symlink_added(&mut self, path: &Path, mount_path: &Path) -> Result<()>;
    fn notify_symlink_changed(&mut self, path: &Path, mount_path: &Path) -> Result<()>;
    fn notify_symlink_removed(&mut self, path: &Path, mount_path: &Path) -> Result<()>;

    fn notify_folder_added(&mut self, path: &Path, mount_path: &Path) -> Result<()>;
    fn notify_folder_changed(&mut self, path: &Path, mount_path: &Path) -> Result<()>;
    fn notify_folder_removed(&mut self, path: &Path, mount_path: &Path) -> Result<()>;
}

#[derive(Serialize, Deserialize)]
pub enum FsNode {
    File,
    Symlink,
    Folder(HashMap<PathBuf, FsNodeType>),
}

#[derive(Serialize, Deserialize)]
pub enum FsNodeType {
    File,
    Symlink,
    Folder,
}

impl FsNode {
    fn node_type(&self) -> FsNodeType {
        match self {
            FsNode::File => FsNodeType::File,
            FsNode::Symlink => FsNodeType::Symlink,
            FsNode::Folder(_) => FsNodeType::Folder
        }
    }
}


pub struct ChangeNotifier<W: FsChangeWatcher> {
    watcher: W
}

impl<W: FsChangeWatcher> ChangeNotifier<W> {
    pub fn new(watcher: W) -> Self {
        Self { watcher }
    }
}

impl<W: FsChangeWatcher> FsProcessor for ChangeNotifier<W> {
    type Item = FsNode;

    fn process_file(&mut self, path: &Path, mount_path: &Path, previous: Option<Self::Item>) -> Result<Self::Item> {
        match previous {
            Some(FsNode::File) => {
                self.watcher.notify_file_changed(path, mount_path)?;
            }
            Some(FsNode::Symlink) => {
                self.watcher.notify_symlink_removed(path, mount_path)?;
                self.watcher.notify_file_added(path, mount_path)?;
            }
            Some(FsNode::Folder(_)) => {
                self.watcher.notify_folder_removed(path, mount_path)?;
                self.watcher.notify_file_added(path, mount_path)?;
            }
            None => {
                self.watcher.notify_file_added(path, mount_path)?;
            }
        }
        Ok(FsNode::File)
    }

    fn process_symlink(&mut self, path: &Path, mount_path: &Path, previous: Option<Self::Item>) -> Result<Self::Item> {
        match previous {
            Some(FsNode::File) => {
                self.watcher.notify_file_removed(path, mount_path)?;
                self.watcher.notify_symlink_added(path, mount_path)?;
            }
            Some(FsNode::Symlink) => {
                self.watcher.notify_symlink_changed(path, mount_path)?;
            }
            Some(FsNode::Folder(_)) => {
                self.watcher.notify_folder_removed(path, mount_path)?;
                self.watcher.notify_symlink_added(path, mount_path)?;
            }
            None => {
                self.watcher.notify_symlink_added(path, mount_path)?;
            }
        }
        Ok(FsNode::Symlink)
    }

    fn process_folder(
        &mut self,
        path: &Path, 
        mount_path: &Path,
        sub: HashMap<PathBuf, FsEntry<Self::Item>>,
        previous: Option<Self::Item>,
    ) -> Result<Self::Item> {
        let new_sub: HashMap<_, _> = sub.into_iter().map(|(k, v)| (k, v.item.node_type())).collect();
        match previous {
            Some(FsNode::File) => {
                self.watcher.notify_file_removed(path, mount_path)?;
                self.watcher.notify_folder_added(path, mount_path)?;
            }
            Some(FsNode::Symlink) => {
                self.watcher.notify_symlink_removed(path, mount_path)?;
                self.watcher.notify_folder_added(path, mount_path)?;
            }
            Some(FsNode::Folder(old_sub)) => {
                let sub_keys: HashSet<_> = new_sub.keys().collect();
                let old_sub_keys: HashSet<_> = old_sub.keys().collect();
                let removed_subs = old_sub_keys.difference(&sub_keys);
                for sub_path in removed_subs {
                    let note_type = old_sub.get(*sub_path).unwrap();
                    let full_sub_path = path.join(sub_path);
                    let new_mount_path = mount_path.join(sub_path);
                    match note_type {
                        FsNodeType::File => {
                            self.watcher.notify_file_removed(&full_sub_path, &new_mount_path)?;
                        }
                        FsNodeType::Symlink => {
                            self.watcher.notify_symlink_removed(&full_sub_path, &new_mount_path)?;
                        }
                        FsNodeType::Folder => {
                            self.watcher.notify_folder_removed(&full_sub_path, &new_mount_path)?;
                        }
                    }
                }
                self.watcher.notify_folder_changed(path, mount_path)?;
            }
            None => {
                self.watcher.notify_folder_added(path, mount_path)?;
            }
        }
        Ok(FsNode::Folder(new_sub))
    }

}
