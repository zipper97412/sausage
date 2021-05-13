use std::{fs::File, path::Path, thread::sleep, time::Duration};

use rusqlite::Connection;
use sausage::{MemoizedFsWalker, TarProcessor, rollback_before_session_id};

mod common;
use common::*;

use anyhow::Result;

fn run_memoized_walker(
    db: &mut Connection,
    path: impl AsRef<Path>,
    tar_path: impl AsRef<Path>,
) -> Result<u32> {
    let path = path.as_ref();
    let mut tar_file = File::create(tar_path)?;
    let proc = TarProcessor::new(&mut tar_file);
    let tx = db.transaction()?;
    let walker = MemoizedFsWalker::new(&*tx);
    let mut adder = walker.start_processing(proc)?;
    let _ = adder.add_path(path, path.file_name().unwrap())?;
    let (_, session_id) = adder.finish_processing()?;
    tx.commit()?;
    Ok(session_id)
}

#[test]
fn test_many_run() -> Result<()> {
    let tmpdir = new_tmpdir("test_many_run")?;
    let mut db = new_sqlite_cache(&tmpdir, "cache.db")?;

    let testdir = new_asset_full(&tmpdir, "asset")?;

    run_memoized_walker(&mut db, &testdir, tmpdir.path().join("testing-full.tar"))?;

    run_memoized_walker(&mut db, &testdir, tmpdir.path().join("testing-empty.tar"))?;

    update_asset_full_1(&testdir)?;

    sleep(Duration::from_secs(1)); //wait for FS modification to be visible, we are too fast!

    let id = run_memoized_walker(&mut db, &testdir, tmpdir.path().join("testing-diff.tar"))?;

    rollback_before_session_id(&db, id)?;

    run_memoized_walker(&mut db, &testdir, tmpdir.path().join("testing-diff-rollback.tar"))?;

    // CHECK\

    Ok(())
}
