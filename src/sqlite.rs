use anyhow::Result;
use rusqlite::{
    params,
    types::{FromSql, FromSqlError, FromSqlResult, ToSqlOutput, Value, ValueRef},
    Connection, ToSql,
};
use std::{
    path::Path,
    time::{Duration, SystemTime},
};

use crate::MemoizedFsCache;

use super::{FsEntry, MemoizedFsCacheSession};

pub struct SqliteSycnSession<'c> {
    db: &'c Connection,
    session_id: u32,
}

impl<'c, I: ToSql + FromSql> MemoizedFsCache<I> for &'c Connection {
    type Session = SqliteSycnSession<'c>;

    fn start_session(self) -> Result<Self::Session> {
        self.execute(
            r#"
        CREATE TEMP TABLE fs_walker_last_session_seen_rows (
            cache_row ROWID NOT NULL PRIMARY KEY
        );
        "#,
            [],
        )?;

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
        Op: FnOnce(Option<I>) -> Result<I>,
    {
        use rusqlite::OptionalExtension;

        let mut stmt = self.db.prepare_cached(
            r#"
        SELECT item, mtime_sec, mtime_nano, rowid FROM fs_walker_cache WHERE path = ?1
        "#,
        )?;
        let sql_path = path.to_string_lossy();
        let sql_mtime_duration = mtime.duration_since(SystemTime::UNIX_EPOCH)?;
        let sql_mtime_sec = sql_mtime_duration.as_secs();
        let sql_mtime_nano = sql_mtime_duration.subsec_nanos();
        let opt_entry = stmt
            .query_row(params![sql_path], |row| {
                let item = row.get(0)?;
                let mtime_sec = row.get(1)?;
                let mtime_nano = row.get(2)?;
                let mtime_dur = Duration::new(mtime_sec, mtime_nano);
                let mtime = SystemTime::UNIX_EPOCH + mtime_dur;
                Ok((FsEntry { mtime, item }, row.get(3)?))
            })
            .optional()?;
        let out;
        let row_id;
        match opt_entry {
            Some((entry, row)) if entry.mtime == mtime => {
                //entry has not changed, retrun back the item
                out = entry;
                row_id = row;
            }
            Some((entry, row)) => {
                let item = compute_item(Some(entry.item))?;
                let mut stmt = self.db.prepare_cached(r#"
                UPDATE fs_walker_cache SET mtime_sec = ?2, mtime_nano = ?3, session_id = ?4, item = ?5 WHERE path = ?1
                "#)?;

                stmt.execute(params![
                    sql_path,
                    sql_mtime_sec,
                    sql_mtime_nano,
                    self.session_id,
                    item
                ])?;
                out = FsEntry { item, mtime };
                row_id = row;
            }
            None => {
                let item = compute_item(None)?;
                let mut stmt = self.db.prepare_cached(r#"
                INSERT INTO fs_walker_cache (path, mtime_sec, mtime_nano, session_id, item) VALUES(?1, ?2, ?3, ?4, ?5)
                "#)?;

                stmt.execute(params![
                    sql_path,
                    sql_mtime_sec,
                    sql_mtime_nano,
                    self.session_id,
                    item
                ])?;
                out = FsEntry { item, mtime };
                row_id = self.db.last_insert_rowid();
            }
        }
        let mut stmt = self.db.prepare_cached(
            r#"
        INSERT INTO fs_walker_last_session_seen_rows (cache_row) VALUES(?1)
        "#,
        )?;
        stmt.execute(params![row_id])?;
        Ok(out)
    }

    fn end_session(self) -> Result<Self::Cache> {
        self.db.execute(
            r#"
        DELETE FROM fs_walker_cache WHERE rowid NOT IN (
            SELECT fs_walker_last_session_seen_rows.cache_row 
            FROM fs_walker_last_session_seen_rows LEFT JOIN fs_walker_cache 
            ON fs_walker_cache.rowid=fs_walker_last_session_seen_rows.cache_row)
        "#,
            [],
        )?;

        self.db.execute(
            r#"
        DROP TABLE fs_walker_last_session_seen_rows;
        "#,
            [],
        )?;
        Ok(self.db)
    }

    fn get_id(&self) -> u32 {
        self.session_id
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

pub fn rollback_before_session_id(db: &Connection, id: u32) -> Result<()> {
    let mut stmt = db.prepare(
        r#"
    DELETE FROM fs_walker_cache WHERE session_id >= ?
    "#,
    )?;
    stmt.execute(params![id])?;

    let mut stmt = db.prepare(
        r#"
    DELETE FROM fs_walker_sessions WHERE session_id >= ?
    "#,
    )?;
    stmt.execute(params![id])?;
    Ok(())
}

impl ToSql for crate::change_watcher::FsNode {
    fn to_sql(&self) -> std::result::Result<ToSqlOutput<'_>, rusqlite::Error> {
        #[cfg(feature = "sqlite_debug")]
        {
            let binary = serde_json::to_string_pretty(self)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
            Ok(ToSqlOutput::Owned(Value::Blob(binary.into_bytes())))
        }
        #[cfg(not(feature = "sqlite_debug"))]
        {
            let binary =
                bincode::serialize(self).map_err(|e| rusqlite::Error::ToSqlConversionFailure(e))?;
            Ok(ToSqlOutput::Owned(Value::Blob(binary)))
        }
    }
}

impl FromSql for crate::change_watcher::FsNode {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value {
            ValueRef::Blob(bytes) => {
                #[cfg(feature = "sqlite_debug")]
                {
                    let node = serde_json::from_slice(bytes)
                        .map_err(|e| FromSqlError::Other(Box::new(e)))?;
                    Ok(node)
                }
                #[cfg(not(feature = "sqlite_debug"))]
                {
                    let node = bincode::deserialize(bytes).map_err(|e| FromSqlError::Other(e))?;
                    Ok(node)
                }
            }
            _ => Err(FromSqlError::InvalidType),
        }
    }
}
