//! SQLite-backed watch log store.
//!
//! Two tables hold the product's data: `films` carries one row per film
//! (TMDB metadata plus the user's own favorites/watchlist state) and
//! `watch_log` carries one JSON document per viewing, with a denormalized
//! `film_id` and `watched_at_ms` so rows can be filtered and ordered without
//! parsing every document.
//!
//! `rewatch` and `watch_count` are projections, not stored user data: they are
//! recomputed on every read from the film's viewings in chronological order
//! (`watched_at_ms`, then row id), so deleting a viewing corrects the rest.

use std::collections::{BTreeMap, HashMap};
use std::path::Path;
use std::time::Duration;

use chrono::{DateTime, Datelike, Utc};
use rusqlite::backup::Backup;
use rusqlite::{Connection, OpenFlags, OptionalExtension, params};
use serde::Serialize;

use crate::domain::{FilmId, FilmListEntry, FilmMetadata, Format, Settings, Venue, WatchLog};
use crate::error::Error;

/// The `settings` table row key for the [`Settings`] document.
const SETTINGS_KEY: &str = "settings";

/// The schema version this build creates and understands.
const SCHEMA_VERSION: i64 = 3;

/// The `films` columns, in [`RawFilmRow`] order.
const FILM_COLUMNS: &str = "film_id, tmdb_id, imdb_id, title, original_title, \
     release_date, release_year, runtime_minutes, overview, tagline, \
     original_language, poster_path, backdrop_path, vote_average, vote_count, \
     popularity, adult, genres, \"cast\", certifications, collection, \
     favorite, watchlisted, metadata_refreshed_at";

/// [`FILM_COLUMNS`] qualified with the `f` alias (for joins).
const FILM_COLUMNS_F: &str = "f.film_id, f.tmdb_id, f.imdb_id, f.title, \
     f.original_title, f.release_date, f.release_year, f.runtime_minutes, \
     f.overview, f.tagline, f.original_language, f.poster_path, \
     f.backdrop_path, f.vote_average, f.vote_count, f.popularity, f.adult, \
     f.genres, f.\"cast\", f.certifications, f.collection, f.favorite, \
     f.watchlisted, f.metadata_refreshed_at";

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
    pub fn open(path: impl AsRef<Path>) -> Result<Self, Error> {
        let conn = Connection::open(path)?;
        configure(&conn)?;
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
        configure(&conn)?;
        let store = Self { conn };
        store.migrate()?;
        Ok(store)
    }

    /// Insert a log, assigning its row id. Returns the stored log, hydrated
    /// from the `films` table with its derived fields.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Sqlite`] on a write failure, [`Error::Serde`] if the
    /// log cannot be serialized, or a validation error if the log is invalid.
    pub fn insert(&self, mut log: WatchLog) -> Result<WatchLog, Error> {
        log.validate()?;
        log.id = 0;
        log.set_derived();
        let watched_at_ms = log.watched_at.timestamp_millis();
        let tx = self.conn.unchecked_transaction()?;
        insert_film_row(
            &tx,
            &FilmMetadata::from_log(&log),
            &Utc::now().to_rfc3339(),
            true,
        )?;
        tx.execute(
            "INSERT INTO watch_log (film_id, watched_at_ms, doc) VALUES (?1, ?2, ?3)",
            params![log.film_id.key(), watched_at_ms, String::new()],
        )?;
        log.id = tx.last_insert_rowid();
        let doc = serde_json::to_string(&log)?;
        tx.execute(
            "UPDATE watch_log SET doc = ?1 WHERE id = ?2",
            params![doc, log.id],
        )?;
        tx.commit()?;
        self.get(log.id)?.ok_or(Error::LogNotFound { id: log.id })
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
        let Some(doc) = doc else {
            return Ok(None);
        };
        let logs = self.hydrate(vec![decode(id, &doc)?])?;
        Ok(logs.into_iter().next())
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
            .prepare("SELECT id, doc FROM watch_log ORDER BY watched_at_ms DESC, id DESC")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })?;
        let mut logs = Vec::new();
        for row in rows {
            let (id, doc) = row?;
            logs.push(decode(id, &doc)?);
        }
        self.hydrate(logs)
    }

    /// The viewings of one film, most recent first.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Sqlite`] on a read failure, or [`Error::Serde`] if a
    /// stored document cannot be deserialized.
    pub fn film_logs(&self, film_id: &FilmId) -> Result<Vec<WatchLog>, Error> {
        let mut stmt = self.conn.prepare(
            "SELECT id, doc FROM watch_log WHERE film_id = ?1 ORDER BY watched_at_ms DESC, id DESC",
        )?;
        let rows = stmt.query_map(params![film_id.key()], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })?;
        let mut logs = Vec::new();
        for row in rows {
            let (id, doc) = row?;
            logs.push(decode(id, &doc)?);
        }
        self.hydrate(logs)
    }

    /// Delete a log by its row id. Returns whether a row was deleted.
    ///
    /// The remaining viewings of the film are recounted on the next read.
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

    /// Update a log by its row id. The supplied `rewatch`/`watch_count` are
    /// ignored and recomputed on read.
    ///
    /// # Errors
    ///
    /// Returns [`Error::LogNotFound`] if `log.id` is not stored,
    /// [`Error::Sqlite`] on a write failure, or a validation error if the
    /// log is invalid.
    pub fn update(&self, mut log: WatchLog) -> Result<WatchLog, Error> {
        if self.get(log.id)?.is_none() {
            return Err(Error::LogNotFound { id: log.id });
        }
        log.validate()?;
        log.set_derived();
        let watched_at_ms = log.watched_at.timestamp_millis();
        let doc = serde_json::to_string(&log)?;
        let tx = self.conn.unchecked_transaction()?;
        insert_film_row(
            &tx,
            &FilmMetadata::from_log(&log),
            &Utc::now().to_rfc3339(),
            true,
        )?;
        tx.execute(
            "UPDATE watch_log SET film_id = ?1, watched_at_ms = ?2, doc = ?3 WHERE id = ?4",
            params![log.film_id.key(), watched_at_ms, doc, log.id],
        )?;
        tx.commit()?;
        self.get(log.id)?.ok_or(Error::LogNotFound { id: log.id })
    }

    /// Insert or update a film's TMDB-sourced metadata.
    ///
    /// The user-owned columns (`favorite`, `watchlisted`) and
    /// `created_at` are preserved on update, so a refresh never loses them.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Sqlite`] on a write failure, or [`Error::Serde`] if a
    /// JSON column cannot be serialized.
    pub fn upsert_film(&self, film: &FilmMetadata) -> Result<(), Error> {
        let updated = upsert_film_row(&self.conn, film)?;
        if updated == 0 {
            return Err(Error::FilmNotFound);
        }
        Ok(())
    }

    /// Get a film's stored metadata, if present.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Sqlite`] on a read failure, or [`Error::Serde`] if a
    /// JSON column cannot be deserialized.
    pub fn get_film(&self, film_id: &FilmId) -> Result<Option<FilmMetadata>, Error> {
        let raw = self
            .conn
            .query_row(
                &format!("SELECT {FILM_COLUMNS} FROM films WHERE film_id = ?1"),
                params![film_id.key()],
                RawFilmRow::from_row,
            )
            .optional()?;
        raw.map(film_from_raw).transpose()
    }

    /// All films in the library, with their viewing summaries, ordered by
    /// title.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Sqlite`] on a read failure, or [`Error::Serde`] if a
    /// JSON column cannot be deserialized.
    pub fn list_films(&self) -> Result<Vec<FilmListEntry>, Error> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {FILM_COLUMNS_F}, COUNT(w.id), MAX(w.watched_at_ms) \
             FROM films f LEFT JOIN watch_log w ON w.film_id = f.film_id \
             GROUP BY f.film_id ORDER BY f.title COLLATE NOCASE"
        ))?;
        let rows = stmt.query_map([], |row| {
            Ok((
                RawFilmRow::from_row(row)?,
                row.get::<_, i64>(FILM_COLUMN_COUNT)?,
                row.get::<_, Option<i64>>(FILM_COLUMN_COUNT + 1)?,
            ))
        })?;
        let mut films = Vec::new();
        for row in rows {
            let (raw, viewings, last) = row?;
            films.push(FilmListEntry {
                film: film_from_raw(raw)?,
                viewings: usize::try_from(viewings).unwrap_or(usize::MAX),
                last_watched_at: last.and_then(DateTime::from_timestamp_millis),
            });
        }
        Ok(films)
    }

    /// Mark a film a favorite (or clear it).
    ///
    /// # Errors
    ///
    /// Returns [`Error::FilmNotFound`] if the film is not stored, otherwise
    /// [`Error::Sqlite`].
    pub fn set_favorite(&self, film_id: &FilmId, favorite: bool) -> Result<(), Error> {
        let n = self.conn.execute(
            "UPDATE films SET favorite = ?2 WHERE film_id = ?1",
            params![film_id.key(), i64::from(favorite)],
        )?;
        if n == 0 {
            return Err(Error::FilmNotFound);
        }
        Ok(())
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
        let mut by_month = BTreeMap::new();
        let mut by_family = BTreeMap::new();
        let mut by_screen = BTreeMap::new();
        let mut by_medium = BTreeMap::new();
        let mut by_flag = BTreeMap::new();
        let mut per_film: HashMap<FilmId, (String, usize)> = HashMap::new();
        let mut rewatches = 0;
        for log in &logs {
            match log.venue {
                Venue::Theatre => by_venue.theatre += 1,
                Venue::Home => by_venue.home += 1,
            }
            *by_year.entry(log.watched_at.year()).or_default() += 1;
            *by_month
                .entry(format!(
                    "{:04}-{:02}",
                    log.watched_at.year(),
                    log.watched_at.month()
                ))
                .or_default() += 1;
            match &log.format {
                Some(Format::Theatre {
                    projection,
                    screen,
                    three_d,
                    hfr,
                    dbox,
                }) => {
                    *by_family
                        .entry(projection.family_key().to_string())
                        .or_default() += 1;
                    *by_screen.entry(screen.key().to_string()).or_default() += 1;
                    for (key, on) in [("three_d", *three_d), ("hfr", *hfr), ("dbox", *dbox)] {
                        if on {
                            *by_flag.entry(key.to_string()).or_default() += 1;
                        }
                    }
                }
                Some(Format::Home { medium }) => {
                    *by_family.entry("home".to_string()).or_default() += 1;
                    *by_medium.entry(medium.key().to_string()).or_default() += 1;
                }
                None => {}
            }
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
            viewings_by_month: by_month,
            viewings_by_projection_family: by_family,
            viewings_by_screen: by_screen,
            viewings_by_medium: by_medium,
            viewings_by_flag: by_flag,
            rewatches,
            top_films,
        })
    }

    /// The database file path this store is open on (`:memory:` for
    /// in-memory stores).
    #[must_use]
    pub fn path(&self) -> &Path {
        match self.conn.path() {
            Some(path) => Path::new(path),
            None => Path::new(":memory:"),
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
    pub fn set_selected_board(&self, board: &str) -> Result<(), Error> {
        let mut settings = self.settings()?;
        board.clone_into(&mut settings.selected_board);
        let value = serde_json::to_string(&settings)?;
        self.conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![SETTINGS_KEY, value],
        )?;
        Ok(())
    }

    /// Write a consistent copy of the database to `dest` via SQLite's online
    /// backup API. Any existing file at `dest` is replaced.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`] if the destination cannot be removed, otherwise
    /// [`Error::Sqlite`].
    pub fn export_to(&self, dest: impl AsRef<Path>) -> Result<(), Error> {
        let dest = dest.as_ref();
        if dest.exists() {
            std::fs::remove_file(dest)?;
        }
        let mut out = Connection::open(dest)?;
        let backup = Backup::new(&self.conn, &mut out)?;
        backup.run_to_completion(100, Duration::from_millis(10), None)?;
        Ok(())
    }

    /// Replace the database contents with the file at `src`.
    ///
    /// The file is validated first (schema version and integrity check), so a
    /// corrupt or unrelated file never replaces live data.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidImport`] if `src` is the live database, is not
    /// a database, has an unusable schema version, or fails the integrity
    /// check; otherwise [`Error::Sqlite`].
    pub fn import_from(&mut self, src: impl AsRef<Path>) -> Result<(), Error> {
        let src = src.as_ref();
        if let (Ok(a), Ok(b)) = (src.canonicalize(), self.path().canonicalize()) {
            if a == b {
                return Err(Error::InvalidImport {
                    detail: "source is the live database".to_owned(),
                });
            }
        }
        let invalid = |e: rusqlite::Error| Error::InvalidImport {
            detail: e.to_string(),
        };
        let source =
            Connection::open_with_flags(src, OpenFlags::SQLITE_OPEN_READ_ONLY).map_err(invalid)?;
        let version: i64 = source
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .map_err(invalid)?;
        if version == 0 || version > SCHEMA_VERSION {
            return Err(Error::InvalidImport {
                detail: format!("unsupported schema version {version}"),
            });
        }
        let check: String = source
            .query_row("PRAGMA integrity_check", [], |row| row.get(0))
            .map_err(invalid)?;
        if check != "ok" {
            return Err(Error::InvalidImport { detail: check });
        }
        let backup = Backup::new(&source, &mut self.conn)?;
        backup.run_to_completion(100, Duration::from_millis(10), None)?;
        drop(backup);
        configure(&self.conn)?;
        self.migrate()?;
        Ok(())
    }

    /// Apply pending migrations.
    fn migrate(&self) -> Result<(), Error> {
        let version: i64 = self
            .conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))?;
        match version {
            0 => self.create_schema(),
            1 => {
                self.migrate_v1_to_v2()?;
                self.migrate_v2_to_v3()
            }
            2 => self.migrate_v2_to_v3(),
            3 => Ok(()),
            v => Err(Error::SchemaTooNew { version: v }),
        }
    }

    /// Create the current schema and stamp [`SCHEMA_VERSION`].
    fn create_schema(&self) -> Result<(), Error> {
        self.conn.execute_batch(&format!(
            "CREATE TABLE IF NOT EXISTS films (
                film_id TEXT PRIMARY KEY NOT NULL,
                tmdb_id INTEGER,
                imdb_id TEXT,
                title TEXT NOT NULL,
                original_title TEXT,
                release_date TEXT,
                release_year INTEGER,
                runtime_minutes INTEGER,
                overview TEXT,
                tagline TEXT,
                original_language TEXT,
                poster_path TEXT,
                backdrop_path TEXT,
                vote_average REAL,
                vote_count INTEGER,
                popularity REAL,
                adult INTEGER NOT NULL DEFAULT 0,
                genres TEXT NOT NULL DEFAULT '[]',
                \"cast\" TEXT NOT NULL DEFAULT '[]',
                certifications TEXT,
                collection TEXT,
                favorite INTEGER NOT NULL DEFAULT 0,
                watchlisted INTEGER NOT NULL DEFAULT 0,
                metadata_refreshed_at TEXT,
                created_at TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS films_title ON films(title);
            CREATE INDEX IF NOT EXISTS films_tmdb_id ON films(tmdb_id);
            CREATE TABLE IF NOT EXISTS watch_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                film_id TEXT NOT NULL REFERENCES films(film_id) ON DELETE CASCADE,
                watched_at_ms INTEGER NOT NULL,
                doc TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS watch_log_watched_at ON watch_log(watched_at_ms DESC);
            CREATE INDEX IF NOT EXISTS watch_log_film ON watch_log(film_id, watched_at_ms);
            CREATE TABLE IF NOT EXISTS settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);
            PRAGMA user_version = {SCHEMA_VERSION};"
        ))?;
        Ok(())
    }

    /// Migrate a v1 database (one JSON document per row, no `films` table) to
    /// v2, preserving every log.
    fn migrate_v1_to_v2(&self) -> Result<(), Error> {
        self.conn.pragma_update(None, "foreign_keys", "OFF")?;
        let tx = self.conn.unchecked_transaction()?;
        let rows: Vec<(i64, String, String)> = {
            let mut stmt = tx.prepare("SELECT id, watched_at, doc FROM watch_log")?;
            let mapped = stmt.query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })?;
            mapped.collect::<Result<Vec<_>, _>>()?
        };
        tx.execute_batch(
            "CREATE TABLE films (
                film_id TEXT PRIMARY KEY NOT NULL,
                tmdb_id INTEGER,
                imdb_id TEXT,
                title TEXT NOT NULL,
                original_title TEXT,
                release_date TEXT,
                release_year INTEGER,
                runtime_minutes INTEGER,
                overview TEXT,
                tagline TEXT,
                original_language TEXT,
                poster_path TEXT,
                backdrop_path TEXT,
                vote_average REAL,
                vote_count INTEGER,
                popularity REAL,
                adult INTEGER NOT NULL DEFAULT 0,
                genres TEXT NOT NULL DEFAULT '[]',
                \"cast\" TEXT NOT NULL DEFAULT '[]',
                certifications TEXT,
                collection TEXT,
                tags TEXT NOT NULL DEFAULT '[]',
                favorite INTEGER NOT NULL DEFAULT 0,
                watchlisted INTEGER NOT NULL DEFAULT 0,
                metadata_refreshed_at TEXT,
                created_at TEXT NOT NULL
            );
            CREATE TABLE watch_log_v2 (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                film_id TEXT NOT NULL REFERENCES films(film_id) ON DELETE CASCADE,
                watched_at_ms INTEGER NOT NULL,
                doc TEXT NOT NULL
            );",
        )?;
        for (id, watched_at, doc) in rows {
            let log: WatchLog =
                serde_json::from_str(&doc).map_err(|_| Error::InvalidStoredLog { id })?;
            let watched_at_ms = DateTime::parse_from_rfc3339(&watched_at)
                .map_err(|_| Error::InvalidStoredLog { id })?
                .timestamp_millis();
            let film = FilmMetadata::from_log(&log);
            insert_film_row(&tx, &film, &Utc::now().to_rfc3339(), true)?;
            tx.execute(
                "INSERT INTO watch_log_v2 (id, film_id, watched_at_ms, doc) VALUES (?1, ?2, ?3, ?4)",
                params![id, log.film_id.key(), watched_at_ms, doc],
            )?;
        }
        tx.execute_batch(
            "DROP TABLE watch_log;
             ALTER TABLE watch_log_v2 RENAME TO watch_log;
             CREATE INDEX watch_log_watched_at ON watch_log(watched_at_ms DESC);
             CREATE INDEX watch_log_film ON watch_log(film_id, watched_at_ms);
             CREATE INDEX films_title ON films(title);
             CREATE INDEX films_tmdb_id ON films(tmdb_id);",
        )?;
        tx.pragma_update(None, "user_version", 2)?;
        tx.commit()?;
        self.conn.pragma_update(None, "foreign_keys", "ON")?;
        Ok(())
    }

    /// Drop the unused `films.tags` column and stamp v3.
    fn migrate_v2_to_v3(&self) -> Result<(), Error> {
        self.conn.execute_batch(
            "ALTER TABLE films DROP COLUMN tags;
             PRAGMA user_version = 3;",
        )?;
        Ok(())
    }

    /// Every stored film, keyed by [`FilmId::key`].
    fn films_map(&self) -> Result<HashMap<String, FilmMetadata>, Error> {
        let mut stmt = self
            .conn
            .prepare(&format!("SELECT {FILM_COLUMNS} FROM films"))?;
        let rows = stmt.query_map([], RawFilmRow::from_row)?;
        let mut map = HashMap::new();
        for row in rows {
            let film = film_from_raw(row?)?;
            map.insert(film.film_id.key(), film);
        }
        Ok(map)
    }

    /// Each viewing's `(total viewings of its film, chronological ordinal)`.
    fn view_counts(&self) -> Result<HashMap<i64, (i32, i32)>, Error> {
        let mut stmt = self.conn.prepare(
            "SELECT id, \
                    COUNT(*) OVER (PARTITION BY film_id), \
                    ROW_NUMBER() OVER (PARTITION BY film_id ORDER BY watched_at_ms, id) \
             FROM watch_log",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })?;
        let mut map = HashMap::new();
        for row in rows {
            let (id, total, ordinal) = row?;
            map.insert(
                id,
                (
                    i32::try_from(total).unwrap_or(i32::MAX),
                    i32::try_from(ordinal).unwrap_or(i32::MAX),
                ),
            );
        }
        Ok(map)
    }

    /// Overwrite each log's film-derived columns from `films`, refresh its
    /// format summary, and set the store-derived `rewatch`/`watch_count`.
    fn hydrate(&self, logs: Vec<WatchLog>) -> Result<Vec<WatchLog>, Error> {
        if logs.is_empty() {
            return Ok(logs);
        }
        let films = self.films_map()?;
        let counts = self.view_counts()?;
        Ok(logs
            .into_iter()
            .map(|mut log| {
                if let Some(film) = films.get(&log.film_id.key()) {
                    log.title.clone_from(&film.title);
                    log.imdb_id.clone_from(&film.imdb_id);
                    log.tmdb_id = film.tmdb_id;
                    log.release_year = film.release_year;
                    log.poster_path.clone_from(&film.poster_path);
                    log.certifications.clone_from(&film.certifications);
                }
                log.set_derived();
                let (total, ordinal) = counts.get(&log.id).copied().unwrap_or((1, 1));
                log.watch_count = Some(total);
                log.rewatch = ordinal > 1;
                log
            })
            .collect())
    }
}

/// The number of columns in [`FILM_COLUMNS`].
const FILM_COLUMN_COUNT: usize = 24;

/// Apply the store's connection pragmas.
fn configure(conn: &Connection) -> Result<(), Error> {
    conn.busy_timeout(Duration::from_secs(5))?;
    let _: String = conn.query_row("PRAGMA journal_mode=WAL", [], |row| row.get(0))?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    Ok(())
}

/// Deserialize a stored log document, stamping the row id that holds it (the
/// row id is authoritative; a migrated or imported document may carry a stale
/// one).
fn decode(id: i64, doc: &str) -> Result<WatchLog, Error> {
    let mut log: WatchLog = serde_json::from_str(doc)?;
    log.id = id;
    Ok(log)
}

/// The `films` columns read back by [`RawFilmRow`].
struct RawFilmRow {
    film_id: String,
    tmdb_id: Option<i64>,
    imdb_id: Option<String>,
    title: String,
    original_title: Option<String>,
    release_date: Option<String>,
    release_year: Option<i32>,
    runtime_minutes: Option<i32>,
    overview: Option<String>,
    tagline: Option<String>,
    original_language: Option<String>,
    poster_path: Option<String>,
    backdrop_path: Option<String>,
    vote_average: Option<f64>,
    vote_count: Option<i64>,
    popularity: Option<f64>,
    adult: i64,
    genres: String,
    cast: String,
    certifications: Option<String>,
    collection: Option<String>,
    favorite: i64,
    watchlisted: i64,
    metadata_refreshed_at: Option<String>,
}

impl RawFilmRow {
    fn from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            film_id: row.get(0)?,
            tmdb_id: row.get(1)?,
            imdb_id: row.get(2)?,
            title: row.get(3)?,
            original_title: row.get(4)?,
            release_date: row.get(5)?,
            release_year: row.get(6)?,
            runtime_minutes: row.get(7)?,
            overview: row.get(8)?,
            tagline: row.get(9)?,
            original_language: row.get(10)?,
            poster_path: row.get(11)?,
            backdrop_path: row.get(12)?,
            vote_average: row.get(13)?,
            vote_count: row.get(14)?,
            popularity: row.get(15)?,
            adult: row.get(16)?,
            genres: row.get(17)?,
            cast: row.get(18)?,
            certifications: row.get(19)?,
            collection: row.get(20)?,
            favorite: row.get(21)?,
            watchlisted: row.get(22)?,
            metadata_refreshed_at: row.get(23)?,
        })
    }
}

/// Convert a raw `films` row into a [`FilmMetadata`].
fn film_from_raw(raw: RawFilmRow) -> Result<FilmMetadata, Error> {
    Ok(FilmMetadata {
        film_id: FilmId::from_key(&raw.film_id)?,
        tmdb_id: raw.tmdb_id,
        imdb_id: raw.imdb_id,
        title: raw.title,
        original_title: raw.original_title,
        release_date: raw.release_date,
        release_year: raw.release_year,
        runtime_minutes: raw.runtime_minutes,
        overview: raw.overview,
        tagline: raw.tagline,
        original_language: raw.original_language,
        poster_path: raw.poster_path,
        backdrop_path: raw.backdrop_path,
        vote_average: raw.vote_average,
        vote_count: raw.vote_count,
        popularity: raw.popularity,
        adult: raw.adult != 0,
        genres: serde_json::from_str(&raw.genres)?,
        cast: serde_json::from_str(&raw.cast)?,
        certifications: raw
            .certifications
            .as_deref()
            .map(serde_json::from_str)
            .transpose()?,
        collection: raw
            .collection
            .as_deref()
            .map(serde_json::from_str)
            .transpose()?,
        favorite: raw.favorite != 0,
        watchlisted: raw.watchlisted != 0,
        metadata_refreshed_at: raw
            .metadata_refreshed_at
            .as_deref()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|d| d.with_timezone(&Utc)),
    })
}

/// Bind the film columns (plus `created_at`) as `?1..=?25`.
fn film_params(
    film: &FilmMetadata,
    created_at: &str,
) -> Result<Vec<rusqlite::types::Value>, Error> {
    let optional_json = |value: &Option<serde_json::Value>| -> Option<String> {
        value.as_ref().map(ToString::to_string)
    };
    let certifications = film
        .certifications
        .as_ref()
        .map(serde_json::to_value)
        .transpose()?;
    let collection = film
        .collection
        .as_ref()
        .map(serde_json::to_value)
        .transpose()?;
    Ok(vec![
        film.film_id.key().into(),
        film.tmdb_id
            .map_or(rusqlite::types::Value::Null, Into::into),
        film.imdb_id
            .clone()
            .map_or(rusqlite::types::Value::Null, Into::into),
        film.title.clone().into(),
        film.original_title
            .clone()
            .map_or(rusqlite::types::Value::Null, Into::into),
        film.release_date
            .clone()
            .map_or(rusqlite::types::Value::Null, Into::into),
        film.release_year
            .map_or(rusqlite::types::Value::Null, Into::into),
        film.runtime_minutes
            .map_or(rusqlite::types::Value::Null, Into::into),
        film.overview
            .clone()
            .map_or(rusqlite::types::Value::Null, Into::into),
        film.tagline
            .clone()
            .map_or(rusqlite::types::Value::Null, Into::into),
        film.original_language
            .clone()
            .map_or(rusqlite::types::Value::Null, Into::into),
        film.poster_path
            .clone()
            .map_or(rusqlite::types::Value::Null, Into::into),
        film.backdrop_path
            .clone()
            .map_or(rusqlite::types::Value::Null, Into::into),
        film.vote_average
            .map_or(rusqlite::types::Value::Null, Into::into),
        film.vote_count
            .map_or(rusqlite::types::Value::Null, Into::into),
        film.popularity
            .map_or(rusqlite::types::Value::Null, Into::into),
        i64::from(film.adult).into(),
        serde_json::to_string(&film.genres)?.into(),
        serde_json::to_string(&film.cast)?.into(),
        optional_json(&certifications).map_or(rusqlite::types::Value::Null, Into::into),
        optional_json(&collection).map_or(rusqlite::types::Value::Null, Into::into),
        i64::from(film.favorite).into(),
        i64::from(film.watchlisted).into(),
        film.metadata_refreshed_at
            .map(|d| d.to_rfc3339())
            .map_or(rusqlite::types::Value::Null, Into::into),
        created_at.to_owned().into(),
    ])
}

/// Insert a film row, ignoring a conflict when `ignore` is set.
fn insert_film_row(
    conn: &Connection,
    film: &FilmMetadata,
    created_at: &str,
    ignore: bool,
) -> Result<usize, Error> {
    let sql = format!(
        "INSERT {} INTO films ({FILM_COLUMNS}, created_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, \
                 ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25)",
        if ignore { "OR IGNORE" } else { "" }
    );
    let values = film_params(film, created_at)?;
    Ok(conn.execute(&sql, rusqlite::params_from_iter(values))?)
}

/// Insert a film row or update its TMDB-sourced columns, preserving the
/// user-owned columns and `created_at`. Returns the affected row count.
fn upsert_film_row(conn: &Connection, film: &FilmMetadata) -> Result<usize, Error> {
    let sql = format!(
        "INSERT INTO films ({FILM_COLUMNS}, created_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, \
                 ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25) \
         ON CONFLICT(film_id) DO UPDATE SET \
             tmdb_id = excluded.tmdb_id, \
             imdb_id = excluded.imdb_id, \
             title = excluded.title, \
             original_title = excluded.original_title, \
             release_date = excluded.release_date, \
             release_year = excluded.release_year, \
             runtime_minutes = excluded.runtime_minutes, \
             overview = excluded.overview, \
             tagline = excluded.tagline, \
             original_language = excluded.original_language, \
             poster_path = excluded.poster_path, \
             backdrop_path = excluded.backdrop_path, \
             vote_average = excluded.vote_average, \
             vote_count = excluded.vote_count, \
             popularity = excluded.popularity, \
             adult = excluded.adult, \
             genres = excluded.genres, \
             \"cast\" = excluded.\"cast\", \
             certifications = excluded.certifications, \
             collection = excluded.collection, \
             metadata_refreshed_at = excluded.metadata_refreshed_at"
    );
    let created_at = film
        .metadata_refreshed_at
        .map_or_else(|| Utc::now().to_rfc3339(), |d| d.to_rfc3339());
    let values = film_params(film, &created_at)?;
    Ok(conn.execute(&sql, rusqlite::params_from_iter(values))?)
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
    /// Viewings by UTC calendar month (`YYYY-MM`), ascending.
    pub viewings_by_month: BTreeMap<String, usize>,
    /// Viewings by projection family: `dci`, `imax`, `film`, or `home`.
    pub viewings_by_projection_family: BTreeMap<String, usize>,
    /// Viewings by auditorium class.
    pub viewings_by_screen: BTreeMap<String, usize>,
    /// Home viewings by release medium.
    pub viewings_by_medium: BTreeMap<String, usize>,
    /// Viewings per presentation flag: `three_d`, `hfr`, or `dbox`.
    pub viewings_by_flag: BTreeMap<String, usize>,
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
    use crate::domain::{
        Dci, FilmId, Format, HomeMedium, Projection, ScreenClass, Venue, WatchLog,
    };
    use chrono::NaiveDate;

    /// A valid log for tests: a [`WatchLog::now`] with a format attached,
    /// matching the venue.
    fn test_log(film_id: FilmId, title: &str, venue: Venue) -> WatchLog {
        let mut log = WatchLog::now(film_id, title, venue);
        let format = if venue == Venue::Home {
            Format::Home {
                medium: HomeMedium::BluRay,
            }
        } else {
            Format::Theatre {
                projection: Projection::Dci(Dci::Laser4K),
                screen: ScreenClass::Plf,
                three_d: false,
                hfr: false,
                dbox: false,
            }
        };
        log.format = Some(format);
        log
    }

    #[test]
    fn insert_and_get_roundtrip() {
        let store = Store::open_in_memory().unwrap();
        let log = test_log(
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
        let a = test_log(FilmId::Imdb("tt0000001".into()), "First", Venue::Home);
        let b = test_log(FilmId::Imdb("tt0000002".into()), "Second", Venue::Home);
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
        let log = test_log(
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
        insert(test_log(a.clone(), "Film A", Venue::Theatre), 2020);
        insert(test_log(a.clone(), "Film A", Venue::Theatre), 2021);
        insert(test_log(a.clone(), "Film A", Venue::Home), 2021);
        insert(test_log(b.clone(), "Film B", Venue::Home), 2021);
        insert(test_log(b.clone(), "Film B", Venue::Home), 2022);

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
