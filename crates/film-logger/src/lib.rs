//! `film_logger` — core library for the `FilmLogger` products.
//!
//! Records films the user watches (when, in UTC, and where: theatre or home)
//! and resolves/enriches them through the TMDB API. The log persists in a
//! single-file SQLite database.

pub mod catalog;
pub mod domain;
pub mod error;
pub mod service;
pub mod store;
pub mod tmdb;

pub use catalog::{FormatCatalog, FormatLegality, FormatOption, format_catalog};
pub use domain::{
    CastMember, Certifications, CollectionRef, Dci, Film, FilmId, FilmListEntry, FilmMetadata,
    FilmPage, FilmStats, Flag, Format, Genre, HomeMedium, Imax, NewLogDetails, Projection, Rating,
    RatingBoard, ScreenClass, Settings, Venue, WatchLog, rating_board, rating_boards,
};
pub use error::Error;
pub use service::FilmLogger;
pub use store::{FilmViewings, Stats, Store, VenueCounts};
pub use tmdb::{COMPILED_API_KEY, Movie, MovieSummary, SearchPage, TmdbClient};
