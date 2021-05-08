#![allow(unused)]

use std::{
    fs::{create_dir, create_dir_all, remove_dir_all, remove_file, File},
    os::unix::fs::symlink,
    path::{Path, PathBuf},
};

use anyhow::Result;
use rusqlite::Connection;
use sausage::setup_sqlite_cache;
use std::io::Write;
use tempdir::TempDir;

pub fn get_test_trash() -> Result<PathBuf> {
    let path = PathBuf::from("trash");
    if path.exists() {
        remove_dir_all(&path)?;
    }
    create_dir(&path)?;
    Ok(path)
}

pub fn new_tmpdir(prefix: &str) -> Result<TempDir> {
    Ok(TempDir::new_in(get_test_trash()?, prefix)?)
}

pub fn new_sqlite_cache(tmp: &TempDir, name: &str) -> Result<Connection> {
    let path = tmp.path().join(name);
    if path.exists() {
        remove_file(&path)?;
    }
    let db = Connection::open(path)?;
    setup_sqlite_cache(&db)?;
    Ok(db)
}

pub fn new_asset_full(tmp: &TempDir, name: &str) -> Result<PathBuf> {
    let path = tmp.path().join(name);
    if path.exists() {
        remove_dir_all(&path)?;
    }
    create_dir(&path)?;
    create_dir(path.join("d1"))?;
    create_dir(path.join("d2"))?;
    create_dir(path.join("d2/d3"))?;

    File::create(path.join("f1"))?;
    File::create(path.join("f2"))?;
    File::create(path.join("d2/f3"))?;

    symlink(path.join("d2/f3"), path.join("d2/s1"))?;
    Ok(path)
}

pub fn update_asset_full_1(path: &Path) -> Result<()> {
    File::create(path.join("f4"))?;

    {
        let mut f = File::create(path.join("f1"))?;
        writeln!(&mut f, "changed")?;
    }

    remove_dir_all(path.join("d2"))?;

    Ok(())
}
