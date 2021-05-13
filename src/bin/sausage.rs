use std::{fs::{File, read_dir}, io::Write, path::PathBuf};

use anyhow::{Result, anyhow};
use clap::{Clap, AppSettings};
use rusqlite::Connection;
use sausage::{MemoizedFsWalker, TarProcessor, rollback_before_session_id, setup_sqlite_cache};
use flate2::Compression;
use flate2::write::GzEncoder;

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

    /// Add compression to create a '.tar.gz'
    #[clap(short = 'z', long)]
    compress: bool,

    /// compression level 
    #[clap(short = 'l', long, default_value = "6")]
    compress_level: u32,

    /// file/folder to include into the tar file use <local path> or <local path>:<tar path>
    #[clap(parse(try_from_str = parse_input_path))]
    input_paths: Vec<(PathBuf, Option<PathBuf>)>,
}

fn parse_input_path(input_path: &str) -> Result<(PathBuf, Option<PathBuf>)> {
    let mut split = input_path.split(':');
    let path = split.next().ok_or_else(|| anyhow!("input_path is malformed, use <path> or <path>:<mount_path>"))?;
    let path = PathBuf::from(path);
    let mount_path = split.next().map(|p|PathBuf::from(p));
    Ok((path, mount_path))
}

fn run<W: Write>(mut writer: W, mut cache: Connection, opts: &Opts) -> Result<()> {
    let proc = TarProcessor::new(&mut writer);
    
    let tx = cache.transaction()?;

    if let Some(rollback_id) = opts.rollback {
        rollback_before_session_id(&*tx, rollback_id)?;
    }
    let walker = MemoizedFsWalker::new(&*tx);
    let mut adder = walker.start_processing(proc)?;
    for (path, opt_mount_path) in &opts.input_paths {
        
        if let Some(mount_path) = opt_mount_path {
            let path = path.canonicalize()?;
            adder.add_path(path, mount_path)?;
        }
        else {
            if let Some(sub) = path.file_name() {
                let path = path.canonicalize()?;
                adder.add_path(&path, sub)?;
            }
            else {
                //this is a folder, add all sub entries instead
                for sub in read_dir(path)? {
                    let entry = sub?;
                    adder.add_path(entry.path().canonicalize()?, entry.file_name())?;
                }
            }
        }
    }
    let (_walker, session_id) = adder.finish_processing()?;
    tx.commit()?;
    println!("session_id {}", session_id);
    Ok(())
}

fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    let cache = if opts.cache_db.exists() {
        Connection::open(&opts.cache_db)?
    } else {
        let db = Connection::open(&opts.cache_db)?;
        setup_sqlite_cache(&db)?;
        db
    };
    let tar_file = File::create(&opts.output_tar)?;

    if opts.compress {
        let enc = GzEncoder::new(tar_file, Compression::new(opts.compress_level));
        run(enc, cache, &opts)
    } else {
        run(tar_file, cache, &opts)
    }
    
}