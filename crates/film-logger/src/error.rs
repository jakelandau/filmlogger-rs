//! Crate-wide error type.

use thiserror::Error;

use crate::domain::Venue;

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

    /// A viewing format was attached to a log whose venue it does not match.
    #[error("viewing format does not match venue")]
    VenueFormatMismatch {
        /// The venue of the log.
        venue: Venue,
        /// The serialized viewing format that did not match.
        format: String,
    },
}
