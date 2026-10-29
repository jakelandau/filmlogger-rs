//! The `FilmLogger` facade.
//!
//! [`FilmLogger`] bundles the store, user settings, and TMDB client into the
//! single surface the desktop and Android apps drive: adding films (search,
//! TMDB enrichment, logging a viewing), listing and editing logs, board
//! selection, statistics, and exporting/importing the whole database.
//!
//! The database is a single file held in write-ahead logging mode;
//! [`FilmLogger::export_to`] checkpoints it and copies the file, and
//! [`FilmLogger::import_from`] replaces the contents from a copy.

use std::fs;
use std::path::Path;

use crate::domain::{Format, RatingBoard, Settings, Venue, WatchLog};
use crate::error::Error;
use crate::store::{Stats, Store};
use crate::tmdb::{Movie, SearchPage, TmdbClient};

/// The app-level facade over the store, settings, and TMDB client.
pub struct FilmLogger {
    store: Store,
    client: TmdbClient,
}

impl FilmLogger {
    /// Open a logger at `path` using the TMDB key baked in at compile time.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NoCompiledApiKey`] if no key was compiled in,
    /// otherwise store errors.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, Error> {
        let client = TmdbClient::from_compiled_key().ok_or(Error::NoCompiledApiKey)?;
        let store = Store::open(path)?;
        Ok(Self { store, client })
    }

    /// Open a logger at `path` with an explicit TMDB client (e.g. one
    /// pointed at a mock server).
    ///
    /// # Errors
    ///
    /// Returns store errors.
    pub fn open_with(path: impl AsRef<Path>, client: TmdbClient) -> Result<Self, Error> {
        let store = Store::open(path)?;
        Ok(Self { store, client })
    }

    /// The store (for direct access, e.g. tests).
    #[must_use]
    pub fn store(&self) -> &Store {
        &self.store
    }

    /// The TMDB client.
    #[must_use]
    pub fn tmdb(&self) -> &TmdbClient {
        &self.client
    }

    /// Search TMDB for films.
    ///
    /// # Errors
    ///
    /// Returns TMDB or HTTP errors.
    pub fn search(&self, query: &str, page: i32) -> Result<SearchPage, Error> {
        self.client.search_movies(query, page)
    }

    /// Find a film by IMDB id, resolving it to a full TMDB record.
    ///
    /// # Errors
    ///
    /// Returns [`Error::FilmNotFound`] if the film is not on TMDB, or
    /// TMDB/HTTP errors.
    pub fn find_imdb(&self, imdb_id: &str) -> Result<Movie, Error> {
        self.client.find_by_imdb_id(imdb_id)
    }

    /// Log a viewing of a film found by TMDB id, enriching the log from the
    /// film's details and board certifications.
    ///
    /// `format`, if any, must match `venue`; it is validated.
    ///
    /// # Errors
    ///
    /// TMDB or store errors; [`Error::VenueFormatMismatch`] or a format
    /// validation error if `format` is inconsistent.
    pub fn add_film(
        &self,
        tmdb_id: i64,
        venue: Venue,
        format: Option<Format>,
    ) -> Result<WatchLog, Error> {
        let movie = self.client.movie(tmdb_id)?;
        let log = match format {
            Some(format) => WatchLog::from_movie_with_format_now(&movie, venue, format)?,
            None => WatchLog::from_movie_now(&movie, venue),
        };
        self.store.insert(log)
    }

    /// Update a stored log (see [`Store::update`]).
    ///
    /// # Errors
    ///
    /// Returns [`Error::LogNotFound`] if the id is not stored, otherwise
    /// store or validation errors.
    pub fn update(&self, log: WatchLog) -> Result<WatchLog, Error> {
        self.store.update(log)
    }

    /// Delete a stored log. Returns whether a row was deleted.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Sqlite`] on a write failure.
    pub fn delete(&self, id: i64) -> Result<bool, Error> {
        self.store.delete(id)
    }

    /// All logs, most recently watched first.
    ///
    /// # Errors
    ///
    /// Returns store or serialization errors.
    pub fn list(&self) -> Result<Vec<WatchLog>, Error> {
        self.store.list()
    }

    /// Aggregate statistics over all logs.
    ///
    /// # Errors
    ///
    /// Returns store or serialization errors.
    pub fn stats(&self) -> Result<Stats, Error> {
        self.store.stats()
    }

    /// The stored settings (defaults when nothing is stored yet).
    ///
    /// # Errors
    ///
    /// Returns store or serialization errors.
    pub fn settings(&self) -> Result<Settings, Error> {
        self.store.settings()
    }

    /// Persist the selected rating board.
    ///
    /// # Errors
    ///
    /// Returns store or serialization errors.
    pub fn set_selected_board(&self, board: RatingBoard) -> Result<(), Error> {
        self.store.set_selected_board(board)
    }

    /// Export the database to `dest`: checkpoint, then copy the file.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`] if the copy fails, otherwise store errors.
    pub fn export_to(&self, dest: impl AsRef<Path>) -> Result<(), Error> {
        self.store.checkpoint()?;
        fs::copy(self.store.path(), dest)?;
        Ok(())
    }

    /// Replace the database contents with the file at `src`.
    ///
    /// A checkpoint of the current database happens first so its file is
    /// self-contained when overwritten.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`] if the copy fails, otherwise store errors.
    pub fn import_from(&mut self, src: impl AsRef<Path>) -> Result<(), Error> {
        let path = self.store.path().to_owned();
        self.store.checkpoint()?;
        fs::copy(src, &path)?;
        let store = Store::open(&path)?;
        self.store = store;
        Ok(())
    }
}
