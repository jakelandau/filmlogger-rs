//! Crate-wide error type.

use thiserror::Error;

use crate::domain::{FilmId, Flag, Format, Projection, ScreenClass, Venue};

/// Crate-wide error.
#[derive(Error, Debug)]
pub enum Error {
    /// An I/O error occurred.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// A SQLite error occurred.
    #[error("database error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    /// A JSON (de)serialization error occurred.
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    /// An HTTP transport error occurred.
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// The TMDB API returned a non-success status.
    #[error("TMDB API error (HTTP {status}): {message}")]
    Api {
        /// The HTTP status code.
        status: u16,
        /// A human-readable message.
        message: String,
    },
    /// The requested film was not found.
    #[error("film not found")]
    FilmNotFound,

    /// A watch log id was not found in the store.
    #[error("log not found: {id}")]
    LogNotFound {
        /// The offending log id.
        id: i64,
    },

    /// The database schema is newer than this build understands.
    #[error("database schema version {version} is newer than this build supports")]
    SchemaTooNew {
        /// The schema version found in the database.
        version: i64,
    },

    /// No TMDB API key was baked in at compile time.
    #[error("no TMDB API key was compiled in")]
    NoCompiledApiKey,

    /// The required viewing format is missing from the log.
    #[error("a viewing format is required")]
    MissingFormat {
        /// The venue of the log.
        venue: Venue,
    },

    /// A viewing format was attached to a log whose venue it does not match.
    #[error("viewing format {format:?} does not match venue {venue:?}")]
    VenueFormatMismatch {
        /// The venue of the log.
        venue: Venue,
        /// The viewing format that did not match.
        format: Format,
    },

    /// A theatre format's projection cannot be screened in its auditorium class.
    #[error("projection {projection:?} does not run in screen {screen:?}")]
    ProjectionScreenMismatch {
        /// The projection technology.
        projection: Projection,
        /// The auditorium class.
        screen: ScreenClass,
    },

    /// A theatre format's presentation flag is incompatible with its projection
    /// and auditorium class.
    #[error("flag {flag:?} does not match projection {projection:?} in screen {screen:?}")]
    FlagMismatch {
        /// The presentation flag.
        flag: Flag,
        /// The projection technology.
        projection: Projection,
        /// The auditorium class.
        screen: ScreenClass,
    },

    /// A watch log's `watch_count` is below one.
    #[error("watch count {count} is less than 1")]
    InvalidWatchCount {
        /// The offending watch count.
        count: i32,
    },

    /// A film id string could not be parsed.
    #[error("invalid film id: {value}")]
    InvalidFilmId {
        /// The offending key.
        value: String,
    },

    /// A stored log document is corrupt or unreadable.
    #[error("stored log {id} is corrupt")]
    InvalidStoredLog {
        /// The row id of the corrupt document.
        id: i64,
    },

    /// An import file was rejected.
    #[error("import rejected: {detail}")]
    InvalidImport {
        /// Why the file was rejected.
        detail: String,
    },

    /// The film has no TMDB id, so it cannot be refreshed.
    #[error("film {film_id:?} has no TMDB id")]
    NoTmdbId {
        /// The film's canonical id.
        film_id: FilmId,
    },
}
