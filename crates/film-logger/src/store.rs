//! SQLite-backed watch log store.
//!
//! Each row stores a whole [`WatchLog`] as a JSON document, plus a
//! denormalized `watched_at` column so rows can be ordered by time without
//! parsing every document. The stored document is authoritative: on insert
//! the row id is assigned and the document is rewritten so its `id` field
//! matches.

use rusqlite::{Connection, OptionalExtension, params};

use crate::domain::{FilmId, WatchLog};
use crate::error::Error;

/// SQLite-backed watch log store.
pub struct Store {
    conn: Connection,
}

impl Store {
    /// Open (or create) a store at `path`, creating the schema if needed.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Sqlite`] if the database cannot be opened or the
    /// schema cannot be created.
    pub fn open(path: impl AsRef<std::path::Path>) -> Result<Self, Error> {
        let conn = Connection::open(path)?;
        let store = Self { conn };
        store.create_table()?;
        Ok(store)
    }

    /// Open an in-memory store (for tests).
    ///
    /// # Errors
    ///
    /// Returns [`Error::Sqlite`] if the in-memory database cannot be
    /// opened or the schema cannot be created.
    pub fn open_in_memory() -> Result<Self, Error> {
        let conn = Connection::open_in_memory()?;
        let store = Self { conn };
        store.create_table()?;
        Ok(store)
    }

    /// Insert a log, assigning its row id. Returns the stored log.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Sqlite`] on a write failure, or [`Error::Serde`]
    /// if the log cannot be serialized to JSON.
    pub fn insert(&self, mut log: WatchLog) -> Result<WatchLog, Error> {
        let prior = self.count_by_film(&log.film_id)?;
        log.rewatch = prior >= 1;
        log.watch_count = Some(prior + 1);
        log.validate()?;
        log.id = 0;
        let watched_at = log.watched_at.to_rfc3339();
        let doc = serde_json::to_string(&log)?;
        self.conn.execute(
            "INSERT INTO watch_log (watched_at, doc) VALUES (?1, ?2)",
            params![watched_at, doc],
        )?;
        log.id = self.conn.last_insert_rowid();
        let doc = serde_json::to_string(&log)?;
        self.conn.execute(
            "UPDATE watch_log SET doc = ?1 WHERE id = ?2",
            params![doc, log.id],
        )?;
        Ok(log)
    }

    /// Get a log by its row id, if present.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Sqlite`] on a read failure, or [`Error::Serde`]
    /// if the stored document cannot be deserialized.
    pub fn get(&self, id: i64) -> Result<Option<WatchLog>, Error> {
        let doc: Option<String> = self
            .conn
            .query_row(
                "SELECT doc FROM watch_log WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .optional()?;
        Ok(doc.map(|d| serde_json::from_str(&d)).transpose()?)
    }

    /// List all logs, most recently watched first.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Sqlite`] on a read failure, or [`Error::Serde`]
    /// if a stored document cannot be deserialized.
    pub fn list(&self) -> Result<Vec<WatchLog>, Error> {
        let mut stmt = self
            .conn
            .prepare("SELECT doc FROM watch_log ORDER BY watched_at DESC, id DESC")?;
        let docs = stmt.query_map([], |row| row.get::<_, String>(0))?;
        let mut logs = Vec::new();
        for doc in docs {
            let doc = doc?;
            logs.push(serde_json::from_str(&doc)?);
        }
        Ok(logs)
    }

    /// Delete a log by its row id. Returns whether a row was deleted.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Sqlite`] on a write failure.
    pub fn delete(&self, id: i64) -> Result<bool, Error> {
        let n = self
            .conn
            .execute("DELETE FROM watch_log WHERE id = ?1", params![id])?;
        Ok(n > 0)
    }

    /// Count stored logs for `film_id`.
    fn count_by_film(&self, film_id: &FilmId) -> Result<i32, Error> {
        let mut stmt = self.conn.prepare("SELECT doc FROM watch_log")?;
        let docs = stmt.query_map([], |row| row.get::<_, String>(0))?;
        let mut n = 0;
        for doc in docs {
            let doc = doc?;
            let Ok(log) = serde_json::from_str::<WatchLog>(&doc) else {
                continue;
            };
            if log.film_id == *film_id {
                n += 1;
            }
        }
        Ok(n)
    }

    fn create_table(&self) -> Result<(), Error> {
        self.conn.execute_batch(
            r"CREATE TABLE IF NOT EXISTS watch_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                watched_at TEXT NOT NULL,
                doc TEXT NOT NULL
            );",
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Store;
    use crate::domain::{FilmId, Venue, WatchLog};

    #[test]
    fn insert_and_get_roundtrip() {
        let store = Store::open_in_memory().unwrap();
        let log = WatchLog::now(
            FilmId::Imdb("tt0137523".into()),
            "Fight Club",
            Venue::Theatre,
        );
        let stored = store.insert(log).unwrap();
        assert!(stored.id > 0);
        let fetched = store.get(stored.id).unwrap().unwrap();
        assert_eq!(fetched, stored);
    }

    #[test]
    fn list_orders_most_recent_first() {
        let store = Store::open_in_memory().unwrap();
        let a = WatchLog::now(FilmId::Imdb("tt0000001".into()), "First", Venue::Home);
        let b = WatchLog::now(FilmId::Imdb("tt0000002".into()), "Second", Venue::Home);
        let a = store.insert(a).unwrap();
        let b = store.insert(b).unwrap();
        let listed = store.list().unwrap();
        assert_eq!(
            listed,
            vec![b.clone(), a.clone()],
            "most recently inserted should be first"
        );
    }

    #[test]
    fn delete_removes_row() {
        let store = Store::open_in_memory().unwrap();
        let log = WatchLog::now(
            FilmId::Imdb("tt0137523".into()),
            "Fight Club",
            Venue::Theatre,
        );
        let stored = store.insert(log).unwrap();
        assert!(store.delete(stored.id).unwrap());
        assert!(!store.delete(stored.id).unwrap());
        assert!(store.get(stored.id).unwrap().is_none());
    }
}
