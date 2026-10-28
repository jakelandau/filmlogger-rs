//! `film_logger` — core library for the `FilmLogger` products.
//!
//! Records films the user watches (when, in UTC, and where: theatre or home)
//! and resolves/enriches them through the TMDB API. The log persists in a
//! single-file SQLite database.

pub mod domain;
pub mod error;
pub mod store;
pub mod tmdb;

pub use domain::{FilmId, Flag, Rating, Venue, WatchLog};
pub use error::Error;
pub use store::Store;
pub use tmdb::{COMPILED_API_KEY, Movie, MovieSummary, SearchPage, TmdbClient};
