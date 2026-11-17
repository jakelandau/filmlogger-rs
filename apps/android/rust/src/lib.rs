//! The Android FFI crate for `filmlogger-rs`.
//!
//! Exposes the [`filmlogger_rs`] facade to Kotlin via uniffi. The uniffi surface
//! is defined in [`types`] (DTOs) and [`convert`] (conversions); this module
//! wires the [`Logger`] object over the core [`FilmLogger`].
//!
//! The generated Kotlin bindings live in the Android app; regenerate them with
//! `uniffi-bindgen` against this crate's compiled cdylib (see the Android
//! build).

mod convert;
pub mod types;

use std::sync::{Arc, Mutex};

use crate::types::{
    Collection, FilmId, FilmListEntry, FilmMetadata, FilmPage, Format, FormatCatalog, LoggerError,
    Movie, NewLogDetails, RatingBoard, SearchPage, Settings, Stats, Venue, WatchLog,
};
use filmlogger_rs::FilmLogger;

/// The shared logger, behind a mutex because methods may need mutable access
/// (e.g. import).
#[derive(uniffi::Object)]
pub struct Logger {
    inner: Mutex<FilmLogger>,
}

/// `#[uniffi::export]` exposes the methods to Kotlin; the owned `String`
/// parameters are intentional because they arrive across the FFI boundary.
#[uniffi::export]
#[allow(clippy::needless_pass_by_value)]
impl Logger {
    /// Open a logger at `path` using the TMDB key baked in at compile time.
    ///
    /// # Errors
    ///
    /// Returns [`LoggerError::NoCompiledApiKey`] if no key was compiled in,
    /// otherwise store errors.
    #[uniffi::constructor]
    pub fn open(path: String) -> Result<Arc<Self>, LoggerError> {
        let logger = FilmLogger::open(path).map_err(LoggerError::from)?;
        Ok(Arc::new(Self {
            inner: Mutex::new(logger),
        }))
    }

    /// Search TMDB for films.
    ///
    /// # Panics
    ///
    /// Panics if the logger's internal mutex is poisoned.
    ///
    /// # Errors
    ///
    /// Returns TMDB or HTTP errors.
    pub fn search(&self, query: String, page: i32) -> Result<SearchPage, LoggerError> {
        let page = self
            .inner
            .lock()
            .expect("logger poisoned")
            .search(&query, page)
            .map_err(LoggerError::from)?;
        Ok(page.into())
    }

    /// Find a film by IMDB id, resolving it to a full TMDB record.
    ///
    /// # Panics
    ///
    /// Panics if the logger's internal mutex is poisoned.
    ///
    /// # Errors
    ///
    /// Returns [`LoggerError::FilmNotFound`] if the film is not on TMDB, or
    /// TMDB/HTTP errors.
    pub fn find_imdb(&self, imdb_id: String) -> Result<Movie, LoggerError> {
        let movie = self
            .inner
            .lock()
            .expect("logger poisoned")
            .find_imdb(&imdb_id)
            .map_err(LoggerError::from)?;
        Ok(movie.into())
    }

    /// Log a viewing of a film found by TMDB id, enriching it from the film's
    /// details.
    ///
    /// # Panics
    ///
    /// Panics if the logger's internal mutex is poisoned.
    ///
    /// # Errors
    ///
    /// TMDB or store errors; a format error if `format` does not match `venue`.
    pub fn add_film(
        &self,
        tmdb_id: i64,
        venue: Venue,
        format: Format,
        details: NewLogDetails,
    ) -> Result<WatchLog, LoggerError> {
        let details = details.to_core()?;
        let log = self
            .inner
            .lock()
            .expect("logger poisoned")
            .add_film(tmdb_id, venue.into(), format.into(), &details)
            .map_err(LoggerError::from)?;
        Ok(log.into())
    }

    /// Update a stored log.
    ///
    /// # Panics
    ///
    /// Panics if the logger's internal mutex is poisoned.
    ///
    /// # Errors
    ///
    /// Returns [`LoggerError::LogNotFound`] if the id is not stored.
    pub fn update(&self, log: WatchLog) -> Result<WatchLog, LoggerError> {
        let core_log = log.to_core()?;
        let updated = self
            .inner
            .lock()
            .expect("logger poisoned")
            .update(core_log)
            .map_err(LoggerError::from)?;
        Ok(updated.into())
    }

    /// Delete a stored log. Returns whether a row was deleted.
    ///
    /// # Panics
    ///
    /// Panics if the logger's internal mutex is poisoned.
    ///
    /// # Errors
    ///
    /// Returns a database error on a write failure.
    pub fn delete(&self, id: i64) -> Result<bool, LoggerError> {
        self.inner
            .lock()
            .expect("logger poisoned")
            .delete(id)
            .map_err(LoggerError::from)
    }

    /// All logs, most recently watched first.
    ///
    /// # Panics
    ///
    /// Panics if the logger's internal mutex is poisoned.
    ///
    /// # Errors
    ///
    /// Returns a database error on a read failure.
    pub fn list(&self) -> Result<Vec<WatchLog>, LoggerError> {
        let logs = self
            .inner
            .lock()
            .expect("logger poisoned")
            .list()
            .map_err(LoggerError::from)?;
        Ok(logs.into_iter().map(Into::into).collect())
    }

    /// Aggregate statistics over all logs.
    ///
    /// # Panics
    ///
    /// Panics if the logger's internal mutex is poisoned.
    ///
    /// # Errors
    ///
    /// Returns a database error on a read failure.
    pub fn stats(&self) -> Result<Stats, LoggerError> {
        let stats = self
            .inner
            .lock()
            .expect("logger poisoned")
            .stats()
            .map_err(LoggerError::from)?;
        Ok(stats.into())
    }

    /// The stored settings.
    ///
    /// # Panics
    ///
    /// Panics if the logger's internal mutex is poisoned.
    ///
    /// # Errors
    ///
    /// Returns a database error on a read failure.
    pub fn settings(&self) -> Result<Settings, LoggerError> {
        let settings = self
            .inner
            .lock()
            .expect("logger poisoned")
            .settings()
            .map_err(LoggerError::from)?;
        Ok(settings.into())
    }

    /// Every rating board TMDB reports certifications for.
    ///
    /// # Panics
    ///
    /// Panics if the logger's internal mutex is poisoned.
    pub fn rating_boards(&self) -> Vec<RatingBoard> {
        self.inner
            .lock()
            .expect("logger poisoned")
            .rating_boards()
            .into_iter()
            .map(Into::into)
            .collect()
    }

    /// Persist the selected rating board by region code.
    ///
    /// # Panics
    ///
    /// Panics if the logger's internal mutex is poisoned.
    ///
    /// # Errors
    ///
    /// Returns a database error on a write failure.
    pub fn set_selected_board(&self, board: String) -> Result<(), LoggerError> {
        self.inner
            .lock()
            .expect("logger poisoned")
            .set_selected_board(&board)
            .map_err(LoggerError::from)
    }

    /// Export the database to `dest`.
    ///
    /// # Panics
    ///
    /// Panics if the logger's internal mutex is poisoned.
    ///
    /// # Errors
    ///
    /// Returns an I/O or database error on a write failure.
    pub fn export_to(&self, dest: String) -> Result<(), LoggerError> {
        self.inner
            .lock()
            .expect("logger poisoned")
            .export_to(dest)
            .map_err(LoggerError::from)
    }

    /// Replace the database contents from `src`.
    ///
    /// # Panics
    ///
    /// Panics if the logger's internal mutex is poisoned.
    ///
    /// # Errors
    ///
    /// Returns an I/O or database error on a read failure.
    pub fn import_from(&self, src: String) -> Result<(), LoggerError> {
        let mut logger = self.inner.lock().expect("logger poisoned");
        logger.import_from(src).map_err(LoggerError::from)
    }

    /// Log a viewing of an already-stored film (no network).
    ///
    /// # Panics
    ///
    /// Panics if the logger's internal mutex is poisoned.
    ///
    /// # Errors
    ///
    /// Returns [`LoggerError::FilmNotFound`] if the film is not stored, or a
    /// format validation error.
    pub fn log_film(
        &self,
        film_id: FilmId,
        venue: Venue,
        format: Format,
        details: NewLogDetails,
    ) -> Result<WatchLog, LoggerError> {
        let details = details.to_core()?;
        let log = self
            .inner
            .lock()
            .expect("logger poisoned")
            .log_film(&film_id.into(), venue.into(), format.into(), &details)
            .map_err(LoggerError::from)?;
        Ok(log.into())
    }

    /// The currently popular movies.
    ///
    /// # Panics
    ///
    /// Panics if the logger's internal mutex is poisoned.
    ///
    /// # Errors
    ///
    /// Returns TMDB or HTTP errors.
    pub fn popular_movies(&self, page: i32) -> Result<SearchPage, LoggerError> {
        let page = self
            .inner
            .lock()
            .expect("logger poisoned")
            .popular_movies(page)
            .map_err(LoggerError::from)?;
        Ok(page.into())
    }

    /// Re-fetch a stored film's metadata from TMDB.
    ///
    /// # Panics
    ///
    /// Panics if the logger's internal mutex is poisoned.
    ///
    /// # Errors
    ///
    /// Returns [`LoggerError::FilmNotFound`] or [`LoggerError::NoTmdbId`].
    pub fn refresh_film(&self, film_id: FilmId) -> Result<FilmMetadata, LoggerError> {
        let film = self
            .inner
            .lock()
            .expect("logger poisoned")
            .refresh_film(&film_id.into())
            .map_err(LoggerError::from)?;
        Ok(film.into())
    }

    /// The film library, with viewing summaries.
    ///
    /// # Panics
    ///
    /// Panics if the logger's internal mutex is poisoned.
    ///
    /// # Errors
    ///
    /// Returns a database error on a read failure.
    pub fn films(&self) -> Result<Vec<FilmListEntry>, LoggerError> {
        let films = self
            .inner
            .lock()
            .expect("logger poisoned")
            .films()
            .map_err(LoggerError::from)?;
        Ok(films.into_iter().map(Into::into).collect())
    }

    /// Fetch a TMDB collection and its films.
    ///
    /// # Panics
    ///
    /// Panics if the logger's internal mutex is poisoned.
    ///
    /// # Errors
    ///
    /// Returns TMDB or transport errors.
    pub fn collection(&self, collection_id: i64) -> Result<Collection, LoggerError> {
        let collection = self
            .inner
            .lock()
            .expect("logger poisoned")
            .collection(collection_id)
            .map_err(LoggerError::from)?;
        Ok(collection.into())
    }

    /// Fetch and store a TMDB film without logging a viewing.
    ///
    /// # Panics
    ///
    /// Panics if the logger's internal mutex is poisoned.
    ///
    /// # Errors
    ///
    /// Returns TMDB or store errors.
    pub fn ensure_film(&self, tmdb_id: i64) -> Result<FilmMetadata, LoggerError> {
        let film = self
            .inner
            .lock()
            .expect("logger poisoned")
            .ensure_film(tmdb_id)
            .map_err(LoggerError::from)?;
        Ok(film.into())
    }

    /// Add a TMDB film to the watchlist, storing it if needed.
    ///
    /// # Panics
    ///
    /// Panics if the logger's internal mutex is poisoned.
    ///
    /// # Errors
    ///
    /// Returns TMDB or store errors.
    pub fn add_to_watchlist(&self, tmdb_id: i64) -> Result<FilmMetadata, LoggerError> {
        let film = self
            .inner
            .lock()
            .expect("logger poisoned")
            .add_to_watchlist(tmdb_id)
            .map_err(LoggerError::from)?;
        Ok(film.into())
    }

    /// Put a stored film on (or remove it from) the watchlist.
    ///
    /// # Panics
    ///
    /// Panics if the logger's internal mutex is poisoned.
    ///
    /// # Errors
    ///
    /// Returns [`LoggerError::FilmNotFound`] if the film is not stored.
    pub fn set_watchlisted(
        &self,
        film_id: FilmId,
        watchlisted: bool,
    ) -> Result<FilmMetadata, LoggerError> {
        let film = self
            .inner
            .lock()
            .expect("logger poisoned")
            .set_watchlisted(&film_id.into(), watchlisted)
            .map_err(LoggerError::from)?;
        Ok(film.into())
    }

    /// One film's page: metadata, history, and statistics.
    ///
    /// # Panics
    ///
    /// Panics if the logger's internal mutex is poisoned.
    ///
    /// # Errors
    ///
    /// Returns [`LoggerError::FilmNotFound`] if the film is not stored.
    pub fn film_page(&self, film_id: FilmId) -> Result<FilmPage, LoggerError> {
        let page = self
            .inner
            .lock()
            .expect("logger poisoned")
            .film_page(&film_id.into())
            .map_err(LoggerError::from)?;
        Ok(page.into())
    }

    /// Mark a film a favorite (or clear it).
    ///
    /// # Panics
    ///
    /// Panics if the logger's internal mutex is poisoned.
    ///
    /// # Errors
    ///
    /// Returns [`LoggerError::FilmNotFound`] if the film is not stored.
    pub fn set_favorite(
        &self,
        film_id: FilmId,
        favorite: bool,
    ) -> Result<FilmMetadata, LoggerError> {
        let film = self
            .inner
            .lock()
            .expect("logger poisoned")
            .set_favorite(&film_id.into(), favorite)
            .map_err(LoggerError::from)?;
        Ok(film.into())
    }

    /// The viewing-format catalog both apps render.
    ///
    /// # Panics
    ///
    /// Panics if the logger's internal mutex is poisoned.
    pub fn format_catalog(&self) -> FormatCatalog {
        self.inner
            .lock()
            .expect("logger poisoned")
            .format_catalog()
            .into()
    }
}

uniffi::setup_scaffolding!();
