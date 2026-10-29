//! SQLite-backed watch log store.
//!
//! Each row stores a whole [`WatchLog`] as a JSON document, plus a
//! denormalized `watched_at` column so rows can be ordered by time without
//! parsing every document. The stored document is authoritative: on insert
//! the row id is assigned and the document is rewritten so its `id` field
//! matches.

use std::collections::{BTreeMap, HashMap};

use chrono::Datelike;
use rusqlite::{Connection, OptionalExtension, params};
use serde::Serialize;

use crate::domain::{FilmId, RatingBoard, Settings, Venue, WatchLog};
use crate::error::Error;

/// The `settings` table row key for the [`Settings`] document.
const SETTINGS_KEY: &str = "settings";

/// The schema version this build creates and understands.
const SCHEMA_VERSION: i64 = 1;

/// SQLite-backed watch log store.
#[derive(Debug)]
pub struct Store {
    conn: Connection,
}

impl Store {
    /// Open (or create) a store at `path`, migrating the schema if needed.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Sqlite`] if the database cannot be opened or
    /// migrated, or [`Error::SchemaTooNew`] if the database's schema version
    /// is newer than this build understands.
    pub fn open(path: impl AsRef<std::path::Path>) -> Result<Self, Error> {
        let conn = Connection::open(path)?;
        let store = Self { conn };
        store.migrate()?;
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
        store.migrate()?;
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

    /// Aggregate statistics over all stored logs.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Sqlite`] on a read failure, or [`Error::Serde`] if a
    /// stored document cannot be deserialized.
    pub fn stats(&self) -> Result<Stats, Error> {
        let logs = self.list()?;
        let mut by_venue = VenueCounts::default();
        let mut by_year = BTreeMap::new();
        let mut per_film: HashMap<FilmId, (String, usize)> = HashMap::new();
        let mut rewatches = 0;
        for log in &logs {
            match log.venue {
                Venue::Theatre => by_venue.theatre += 1,
                Venue::Home => by_venue.home += 1,
            }
            *by_year.entry(log.watched_at.year()).or_default() += 1;
            if log.rewatch {
                rewatches += 1;
            }
            let entry = per_film
                .entry(log.film_id.clone())
                .or_insert_with(|| (log.title.clone(), 0));
            entry.1 += 1;
        }
        let total_viewings = logs.len();
        let distinct_films = per_film.len();
        let mut top_films: Vec<FilmViewings> = per_film
            .into_iter()
            .map(|(_, (title, count))| FilmViewings { title, count })
            .collect();
        top_films.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.title.cmp(&b.title)));
        Ok(Stats {
            total_viewings,
            distinct_films,
            viewings_by_venue: by_venue,
            viewings_by_year: by_year,
            rewatches,
            top_films,
        })
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

    /// Update a log by its row id, recomputing the store-maintained fields
    /// (`rewatch`, `watch_count`) against the store's current contents.
    ///
    /// # Errors
    ///
    /// Returns [`Error::LogNotFound`] if `log.id` is not stored,
    /// [`Error::Sqlite`] on a write failure, or a validation error if the
    /// log is invalid.
    pub fn update(&self, mut log: WatchLog) -> Result<WatchLog, Error> {
        let Some(existing) = self.get(log.id)? else {
            return Err(Error::LogNotFound { id: log.id });
        };
        let mut prior = self.count_by_film(&log.film_id)?;
        if existing.film_id == log.film_id {
            // The stored row is this log; exclude it from the count.
            prior -= 1;
        }
        log.rewatch = prior >= 1;
        log.watch_count = Some(prior + 1);
        log.validate()?;
        let watched_at = log.watched_at.to_rfc3339();
        let doc = serde_json::to_string(&log)?;
        self.conn.execute(
            "UPDATE watch_log SET watched_at = ?1, doc = ?2 WHERE id = ?3",
            params![watched_at, doc, log.id],
        )?;
        Ok(log)
    }

    /// The database file path this store is open on (`:memory:` for
    /// in-memory stores).
    #[must_use]
    pub fn path(&self) -> &std::path::Path {
        match self.conn.path() {
            Some(path) => std::path::Path::new(path),
            None => std::path::Path::new(":memory:"),
        }
    }

    /// The stored [`Settings`].
    ///
    /// The default when nothing is stored yet.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Sqlite`] on a read failure, or [`Error::Serde`] if
    /// the stored document cannot be deserialized.
    pub fn settings(&self) -> Result<Settings, Error> {
        let value: Option<String> = self
            .conn
            .query_row(
                "SELECT value FROM settings WHERE key = ?1",
                params![SETTINGS_KEY],
                |row| row.get(0),
            )
            .optional()?;
        match value {
            Some(value) => Ok(serde_json::from_str(&value)?),
            None => Ok(Settings::default()),
        }
    }

    /// Persist the selected rating board (other settings keep their stored
    /// values).
    ///
    /// # Errors
    ///
    /// Returns [`Error::Sqlite`] on a write failure, or [`Error::Serde`]
    /// if the settings cannot be serialized to JSON.
    pub fn set_selected_board(&self, board: RatingBoard) -> Result<(), Error> {
        let mut settings = self.settings()?;
        settings.selected_board = board;
        let value = serde_json::to_string(&settings)?;
        self.conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![SETTINGS_KEY, value],
        )?;
        Ok(())
    }

    /// Consolidate any write-ahead log into the main database file.
    ///
    /// After this, the database file may be copied safely (export).
    ///
    /// # Errors
    ///
    /// Returns [`Error::Sqlite`] on a failure.
    pub fn checkpoint(&self) -> Result<(), Error> {
        self.conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE)")?;
        Ok(())
    }

    /// Apply pending migrations for `conn`.
    fn migrate(&self) -> Result<(), Error> {
        let version: i64 = self
            .conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))?;
        if version == 0 {
            self.create_schema()?;
        } else if version > SCHEMA_VERSION {
            return Err(Error::SchemaTooNew { version });
        }
        Ok(())
    }

    /// Create the schema and stamp [`SCHEMA_VERSION`].
    fn create_schema(&self) -> Result<(), Error> {
        self.conn.execute_batch(
            r"CREATE TABLE IF NOT EXISTS watch_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                watched_at TEXT NOT NULL,
                doc TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );",
        )?;
        self.conn
            .pragma_update(None, "user_version", SCHEMA_VERSION)?;
        Ok(())
    }
}

/// Aggregated statistics over all stored watch logs.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Stats {
    /// Total viewings (every watch log).
    pub total_viewings: usize,
    /// Number of distinct films.
    pub distinct_films: usize,
    /// Viewings by venue.
    pub viewings_by_venue: VenueCounts,
    /// Viewings by UTC calendar year, ascending.
    pub viewings_by_year: BTreeMap<i32, usize>,
    /// Viewings flagged as a rewatch.
    pub rewatches: usize,
    /// Top films by viewing count, descending (title as tie-break).
    pub top_films: Vec<FilmViewings>,
}

impl Stats {
    /// Rewatch rate: rewatches over total viewings (`0.0` if none).
    ///
    /// Counts are realistically far below `f64`'s 52-bit mantissa, so the
    /// cast cannot lose precision in practice.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn rewatch_rate(&self) -> f64 {
        if self.total_viewings == 0 {
            0.0
        } else {
            self.rewatches as f64 / self.total_viewings as f64
        }
    }
}

/// Viewings broken down by venue.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct VenueCounts {
    /// Theatrical screenings.
    pub theatre: usize,
    /// Home viewings.
    pub home: usize,
}

/// A single film's viewing count, for ranked reports.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FilmViewings {
    /// Denormalized film title.
    pub title: String,
    /// Number of times this film has been watched.
    pub count: usize,
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{FilmViewings, Store, VenueCounts};
    use crate::domain::{FilmId, Venue, WatchLog};
    use chrono::NaiveDate;

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

    #[test]
    fn stats_empty_store_is_zeroed() {
        let store = Store::open_in_memory().unwrap();
        let stats = store.stats().unwrap();
        assert_eq!(stats.total_viewings, 0);
        assert_eq!(stats.distinct_films, 0);
        assert_eq!(stats.viewings_by_venue, VenueCounts::default());
        assert_eq!(stats.viewings_by_year, BTreeMap::new());
        assert_eq!(stats.rewatches, 0);
        assert_eq!(stats.rewatch_rate(), 0.0);
        assert_eq!(stats.top_films, Vec::new());
    }

    #[test]
    fn stats_aggregate_stored_logs() {
        // film A: 3 viewings (theatre 2020, theatre 2021, home 2021)
        // film B: 2 viewings (home 2021, home 2022)
        let store = Store::open_in_memory().unwrap();
        let a = FilmId::Imdb("tt0000001".into());
        let b = FilmId::Imdb("tt0000002".into());
        let insert = |mut log: WatchLog, year: i32| {
            log.watched_at = NaiveDate::from_ymd_opt(year, 6, 1)
                .unwrap()
                .and_hms_opt(12, 0, 0)
                .unwrap()
                .and_utc();
            store.insert(log).unwrap();
        };
        insert(WatchLog::now(a.clone(), "Film A", Venue::Theatre), 2020);
        insert(WatchLog::now(a.clone(), "Film A", Venue::Theatre), 2021);
        insert(WatchLog::now(a.clone(), "Film A", Venue::Home), 2021);
        insert(WatchLog::now(b.clone(), "Film B", Venue::Home), 2021);
        insert(WatchLog::now(b.clone(), "Film B", Venue::Home), 2022);

        let stats = store.stats().unwrap();
        assert_eq!(stats.total_viewings, 5);
        assert_eq!(stats.distinct_films, 2);
        assert_eq!(
            stats.viewings_by_venue,
            VenueCounts {
                theatre: 2,
                home: 3
            }
        );
        let mut by_year = BTreeMap::new();
        by_year.insert(2020, 1);
        by_year.insert(2021, 3);
        by_year.insert(2022, 1);
        assert_eq!(stats.viewings_by_year, by_year);
        assert_eq!(stats.rewatches, 3);
        assert!((stats.rewatch_rate() - 0.6).abs() < 1e-9);
        assert_eq!(
            stats.top_films,
            vec![
                FilmViewings {
                    title: "Film A".into(),
                    count: 3
                },
                FilmViewings {
                    title: "Film B".into(),
                    count: 2
                },
            ],
        );
    }
}
