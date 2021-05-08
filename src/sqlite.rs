use anyhow::Result;
use rusqlite::{params, types::FromSql, Connection, ToSql};
use std::{path::Path, time::SystemTime};

use crate::MemoizedFsCache;

use super::{FsEntry, MemoizedFsCacheSession};

pub struct SqliteSycnSession<'c> {
    db: &'c Connection,
    session_id: u32,
}

impl<'c, I: ToSql + FromSql> MemoizedFsCache<I> for &'c Connection {
    type Session = SqliteSycnSession<'c>;

    fn start_session(self) -> Result<Self::Session> {
        let mut stmt = self.prepare_cached(
            r#"
        SELECT MAX(session_id) FROM fs_walker_sessions;
        "#,
        )?;
        let opt_max_session_id = stmt.query_row(params![], |row| row.get::<_, Option<_>>(0))?;
        let session_id = opt_max_session_id.unwrap_or(0) + 1;
        let mut stmt = self.prepare_cached(
            r#"
        INSERT INTO fs_walker_sessions (session_id) VALUES(?1)
        "#,
        )?;
        stmt.execute(params![session_id])?;
        Ok(SqliteSycnSession {
            db: self,
            session_id,
        })
    }
}

impl<'c, I: ToSql + FromSql> MemoizedFsCacheSession<I> for SqliteSycnSession<'c> {
    type Cache = &'c Connection;

    fn get_update_entry_from_cache<Op>(
        &mut self,
        path: &Path,
        mtime: SystemTime,
        compute_item: Op,
    ) -> Result<FsEntry<I>>
    where
        Op: FnOnce() -> Result<I>,
    {
        use rusqlite::OptionalExtension;
        let mut stmt = self.db.prepare_cached(
            r#"
        SELECT item FROM fs_walker_cache WHERE path = ?1 AND mtime_sec = ?2 AND mtime_nano = ?3
        "#,
        )?;
        let sql_path = path.to_string_lossy();
        let sql_mtime_duration = mtime.duration_since(SystemTime::UNIX_EPOCH)?;
        let sql_mtime_sec = sql_mtime_duration.as_secs();
        let sql_mtime_nano = sql_mtime_duration.subsec_nanos();
        let opt_entry = stmt
            .query_row(params![sql_path, sql_mtime_sec, sql_mtime_nano], |row| {
                Ok(FsEntry {
                    mtime,
                    item: row.get(0)?,
                })
            })
            .optional()?;
        if let Some(entry) = opt_entry {
            Ok(entry)
        } else {
            let item = compute_item()?;
            let mut stmt = self.db.prepare_cached(r#"
            INSERT INTO fs_walker_cache (path, mtime_sec, mtime_nano, session_id, item) VALUES(?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(path) DO UPDATE SET mtime_sec = excluded.mtime_sec, mtime_nano = excluded.mtime_nano, session_id = excluded.session_id, item = excluded.item
            "#)?;
            stmt.execute(params![
                sql_path,
                sql_mtime_sec,
                sql_mtime_nano,
                self.session_id,
                item
            ])?;
            Ok(FsEntry { item, mtime })
        }
    }

    fn end_session(self) -> Result<Self::Cache> {
        Ok(self.db)
    }
}

pub fn setup_sqlite_cache(db: &Connection) -> Result<()> {
    db.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS fs_walker_sessions (
            session_id INTEGER NOT NULL,
            PRIMARY KEY (session_id)
        );
        CREATE TABLE IF NOT EXISTS fs_walker_cache (
            path TEXT NOT NULL, 
            mtime_sec INTEGER NOT NULL,
            mtime_nano INTEGER NOT NULL,
            session_id INTEGER NOT NULL,
            item BLOB,
            PRIMARY KEY (path),
            FOREIGN KEY (session_id) REFERENCES fs_walker_sessions (session_id) 
        );
    "#,
    )?;
    Ok(())
}
