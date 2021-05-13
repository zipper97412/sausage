use std::{fs::File, path::PathBuf};

use anyhow::Result;
use clap::{Clap, AppSettings};
use rusqlite::Connection;
use sausage::{MemoizedFsWalker, TarProcessor, rollback_before_session_id, setup_sqlite_cache};


/// This doc string acts as a help message when the user runs '--help'
/// as do all doc strings on fields
#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    /// Output tar file that will contain only changed files since the last run
    #[clap(short, long)]
    output_tar: PathBuf,
    /// Cache database to use for this execution, will be updated with a transaction when the tar is generated
    #[clap(short, long)]
    cache_db: PathBuf,
    /// Rollback to a previous session id before execution
    #[clap(short, long)]
    rollback: Option<u32>,

    /// file/folder to include into the tar file
    input_path: PathBuf,
}



fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    let mut cache = if opts.cache_db.exists() {
        Connection::open(opts.cache_db)?
    } else {
        let db = Connection::open(opts.cache_db)?;
        setup_sqlite_cache(&db)?;
        db
    };
    let mut tar_file = File::create(opts.output_tar)?;
    let proc = TarProcessor::new(&mut tar_file);
    
    let tx = cache.transaction()?;

    if let Some(rollback_id) = opts.rollback {
        rollback_before_session_id(&*tx, rollback_id)?;
    }
    let mut walker = MemoizedFsWalker::new(proc);
    let (_, session_id, _) = walker.hash_path(&*tx, opts.input_path)?;
    
    tx.commit()?;
    println!("session_id {}", session_id);
    Ok(())
}