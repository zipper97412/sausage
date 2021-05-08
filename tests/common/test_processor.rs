#![allow(unused)]
use anyhow::Result;
use sausage::{FsEntry, FsProcessor};
use std::io::Write;
use std::{collections::HashMap, ffi::OsString, path::Path};

pub struct TestProcessor<W: Write> {
    acc: W,
}

impl<W: Write> TestProcessor<W> {
    pub fn new(mut acc: W) -> Result<Self> {
        writeln!(&mut acc)?; // to help with r#"..."#
        Ok(TestProcessor { acc })
    }
}

impl<W: Write> FsProcessor for TestProcessor<W> {
    type Item = Option<bool>;

    fn process_file(&mut self, path: &Path) -> Result<Self::Item> {
        writeln!(&mut self.acc, "F|{}", path.to_string_lossy())?;
        Ok(None)
    }

    fn process_symlink(&mut self, path: &Path) -> Result<Self::Item> {
        writeln!(&mut self.acc, "S|{}", path.to_string_lossy())?;
        Ok(None)
    }

    fn process_folder(
        &mut self,
        path: &Path,
        sub: HashMap<OsString, FsEntry<Self::Item>>,
    ) -> Result<Self::Item> {
        writeln!(&mut self.acc, "D|{}|{}", path.to_string_lossy(), sub.len())?;
        Ok(None)
    }
}
