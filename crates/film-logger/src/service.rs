//! The `FilmLogger` facade.
//!
//! [`FilmLogger`] bundles the store, user settings, and TMDB client into the
//! single surface the desktop and Android apps drive: searching films, adding
//! and logging viewings, the film library and per-film pages, board
//! selection, statistics, and exporting/importing the whole database.
//!
//! Export and import go through SQLite's online backup API, so a copy is
//! consistent even while the database runs in write-ahead logging mode, and an
//! imported file is validated before it can replace live data.

use std::path::Path;

use chrono::Local;

use crate::catalog::{FormatCatalog, format_catalog};
use crate::domain::{
    FilmId, FilmListEntry, FilmMetadata, FilmPage, FilmStats, Format, NewLogDetails, RatingBoard,
    Settings, Venue, WatchLog,
};
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

    /// The currently popular movies.
    ///
    /// # Errors
    ///
    /// Returns TMDB or HTTP errors.
    pub fn popular_movies(&self, page: i32) -> Result<SearchPage, Error> {
        self.client.popular_movies(page)
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

    /// Add a film by TMDB id and log a viewing of it, enriching both the
    /// film's stored metadata and the log from TMDB.
    ///
    /// # Errors
    ///
    /// TMDB or store errors; [`Error::VenueFormatMismatch`] or a format
    /// validation error if `format` is inconsistent.
    pub fn add_film(
        &self,
        tmdb_id: i64,
        venue: Venue,
        format: Format,
        details: &NewLogDetails,
    ) -> Result<WatchLog, Error> {
        if format.venue() != venue {
            return Err(Error::VenueFormatMismatch { venue, format });
        }
        format.validate()?;
        let movie = self.client.movie(tmdb_id)?;
        self.store.upsert_film(&FilmMetadata::from_movie(&movie))?;
        let mut log = WatchLog::from_movie_with_format_now(&movie, venue, format)?;
        apply_details(&mut log, details);
        self.store.insert(log)
    }

    /// Log a viewing of an already-stored film. With no network access, the
    /// log's denormalized fields come from the film's stored metadata.
    ///
    /// # Errors
    ///
    /// Returns [`Error::FilmNotFound`] if the film is not stored, a format
    /// validation error if `format` is inconsistent, or store errors.
    pub fn log_film(
        &self,
        film_id: &FilmId,
        venue: Venue,
        format: Format,
        details: &NewLogDetails,
    ) -> Result<WatchLog, Error> {
        let film = self.store.get_film(film_id)?.ok_or(Error::FilmNotFound)?;
        let mut log = WatchLog::from_film(&film, venue, format, Local::now())?;
        apply_details(&mut log, details);
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

    /// The film library, ordered by title.
    ///
    /// # Errors
    ///
    /// Returns store or serialization errors.
    pub fn films(&self) -> Result<Vec<FilmListEntry>, Error> {
        self.store.list_films()
    }

    /// One film's page: metadata, viewing history, and statistics.
    ///
    /// # Errors
    ///
    /// Returns [`Error::FilmNotFound`] if the film is not stored, otherwise
    /// store or serialization errors.
    pub fn film_page(&self, film_id: &FilmId) -> Result<FilmPage, Error> {
        let film = self.store.get_film(film_id)?.ok_or(Error::FilmNotFound)?;
        let logs = self.store.film_logs(film_id)?;
        Ok(FilmPage {
            stats: film_stats(&logs),
            film,
            logs,
        })
    }

    /// Re-fetch a stored film's metadata from TMDB, keeping its canonical id
    /// (so its logs stay attached) and its user-owned state.
    ///
    /// # Errors
    ///
    /// Returns [`Error::FilmNotFound`] if the film is not stored,
    /// [`Error::NoTmdbId`] if it has no TMDB id, otherwise TMDB or store
    /// errors.
    pub fn refresh_film(&self, film_id: &FilmId) -> Result<FilmMetadata, Error> {
        let existing = self.store.get_film(film_id)?.ok_or(Error::FilmNotFound)?;
        let tmdb_id = existing.tmdb_id.ok_or_else(|| Error::NoTmdbId {
            film_id: existing.film_id.clone(),
        })?;
        let movie = self.client.movie(tmdb_id)?;
        let mut film = FilmMetadata::from_movie(&movie);
        film.film_id.clone_from(&existing.film_id);
        self.store.upsert_film(&film)?;
        self.store
            .get_film(&existing.film_id)?
            .ok_or(Error::FilmNotFound)
    }

    /// Mark a film a favorite (or clear it), returning the stored film.
    ///
    /// # Errors
    ///
    /// Returns [`Error::FilmNotFound`] if the film is not stored, otherwise
    /// store errors.
    pub fn set_favorite(&self, film_id: &FilmId, favorite: bool) -> Result<FilmMetadata, Error> {
        self.store.set_favorite(film_id, favorite)?;
        self.store.get_film(film_id)?.ok_or(Error::FilmNotFound)
    }

    /// The viewing-format catalog both apps render.
    #[must_use]
    pub fn format_catalog(&self) -> FormatCatalog {
        format_catalog()
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

    /// Every rating board TMDB reports certifications for.
    #[must_use]
    pub fn rating_boards(&self) -> Vec<RatingBoard> {
        crate::domain::rating_boards()
    }

    /// Persist the selected rating board by region code.
    ///
    /// # Errors
    ///
    /// Returns store or serialization errors.
    pub fn set_selected_board(&self, board: &str) -> Result<(), Error> {
        self.store.set_selected_board(board)
    }

    /// Export the database to `dest` with SQLite's backup API.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`] if an existing destination cannot be removed,
    /// otherwise store errors.
    pub fn export_to(&self, dest: impl AsRef<Path>) -> Result<(), Error> {
        self.store.export_to(dest)
    }

    /// Replace the database contents with the validated file at `src`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidImport`] if the file is rejected, otherwise
    /// store errors.
    pub fn import_from(&mut self, src: impl AsRef<Path>) -> Result<(), Error> {
        self.store.import_from(src)
    }
}

/// Apply the user-supplied details to a new log.
fn apply_details(log: &mut WatchLog, details: &NewLogDetails) {
    if let Some(watched_at) = details.watched_at {
        log.watched_at = watched_at;
    }
    if let Some(timezone) = &details.timezone {
        let timezone = timezone.trim();
        log.timezone = if timezone.is_empty() {
            None
        } else {
            Some(timezone.to_owned())
        };
    }
    if let Some(notes) = &details.notes {
        let notes = notes.trim();
        log.notes = if notes.is_empty() {
            None
        } else {
            Some(notes.to_owned())
        };
    }
    if let Some(rating) = details.rating {
        log.rating = Some(rating);
    }
}

/// Compute a film's viewing statistics from its logs (most recent first).
fn film_stats(logs: &[WatchLog]) -> FilmStats {
    let mut stats = FilmStats {
        viewings: logs.len(),
        ..FilmStats::default()
    };
    for log in logs {
        match log.venue {
            Venue::Theatre => stats.theatre += 1,
            Venue::Home => stats.home += 1,
        }
        *stats.formats.entry(log.format_summary.clone()).or_default() += 1;
    }
    stats.first_watched_at = logs.iter().map(|log| log.watched_at).min();
    stats.last_watched_at = logs.iter().map(|log| log.watched_at).max();
    stats
}
