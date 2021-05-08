use std::{fs::File, path::Path};

use rusqlite::Connection;
use sausage::{MemoizedFsWalker, TarProcessor};

mod common;
use common::*;

use anyhow::Result;

fn run_memoized_walker(
    db: &mut Connection,
    path: impl AsRef<Path>,
    tar_path: impl AsRef<Path>,
) -> Result<()> {
    let mut tar_file = File::create(tar_path)?;
    let proc = TarProcessor::new(&mut tar_file);
    let tx = db.transaction()?;
    let mut walker = MemoizedFsWalker::new(proc);
    let _ = walker.hash_path(&*tx, path)?;
    tx.commit()?;
    Ok(())
}

#[test]
fn test_many_run() -> Result<()> {
    let tmpdir = new_tmpdir("test_many_run")?;
    let mut db = new_sqlite_cache(&tmpdir, "cache.db")?;

    let testdir = new_asset_full(&tmpdir, "asset")?;

    run_memoized_walker(&mut db, &testdir, tmpdir.path().join("testing-full.tar"))?;

    run_memoized_walker(&mut db, &testdir, tmpdir.path().join("testing-empty.tar"))?;

    update_asset_full_1(&testdir)?;

    run_memoized_walker(&mut db, &testdir, tmpdir.path().join("testing-diff.tar"))?;

    // CHECK\

    Ok(())
}
