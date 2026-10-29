//! `film_logger` — core library for the `FilmLogger` products.
//!
//! Records films the user watches (when, in UTC, and where: theatre or home)
//! and resolves/enriches them through the TMDB API. The log persists in a
//! single-file SQLite database.

pub mod domain;
pub mod error;
pub mod service;
pub mod store;
pub mod tmdb;

pub use domain::{
    Certifications, Dci, Film, FilmId, Flag, Format, HomeMedium, Imax, Projection, Rating,
    RatingBoard, ScreenClass, Settings, Venue, WatchLog,
};
pub use error::Error;
pub use service::FilmLogger;
pub use store::{FilmViewings, Stats, Store, VenueCounts};
pub use tmdb::{COMPILED_API_KEY, Genre, Movie, MovieSummary, SearchPage, TmdbClient};
