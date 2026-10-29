//! Uniffi DTO types for the Android FFI surface.
//!
//! These mirror [`film_logger`]'s domain types, but restricted to uniffi's
//! supported data shapes (no `DateTime<Utc>`, `usize`, or `BTreeMap`).
//! `watched_at` is carried as an RFC 3339 string, and `viewings_by_year` as a
//! vector of [`YearCount`].

/// Where a film was watched.
#[derive(uniffi::Enum, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Venue {
    /// A theatrical screening.
    Theatre,
    /// A home viewing.
    Home,
}

/// A thumbs-up or thumbs-down rating of a viewing.
#[derive(uniffi::Enum, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Rating {
    /// Thumbs up.
    Up,
    /// Thumbs down.
    Down,
}

/// The board whose certification is shown as a film's content rating.
#[derive(uniffi::Enum, Clone, Copy, PartialEq, Eq, Debug)]
pub enum RatingBoard {
    /// The US MPAA rating board.
    Mpa,
    /// The British Board of Film Classification.
    Bbfc,
    /// The German FSK board.
    Fsk,
}

/// A canonical film identifier: an IMDB id, or a TMDB numeric id when the
/// film has no IMDB id.
#[derive(uniffi::Enum, Clone, PartialEq, Eq, Debug)]
pub enum FilmId {
    /// An IMDB id, e.g. `"tt0137523"`.
    Imdb { id: String },
    /// A TMDB numeric id.
    Tmdb { id: i64 },
}

/// How a film was viewed. A [`Format::Theatre`] is a projection technology
/// and auditorium class; a [`Format::Home`] is a release medium.
#[derive(uniffi::Enum, Clone, PartialEq, Eq, Debug)]
pub enum Format {
    /// A theatrical screening.
    Theatre {
        /// The projection technology.
        projection: Projection,
        /// The auditorium class.
        screen: ScreenClass,
        /// Presented in 3D.
        three_d: bool,
        /// Presented at high frame rate.
        hfr: bool,
        /// Presented on D-BOX motion seats.
        dbox: bool,
    },
    /// A home viewing.
    Home {
        /// The release medium.
        medium: HomeMedium,
    },
}

/// Theatrical projection technology, grouped by projection family.
#[derive(uniffi::Enum, Clone, PartialEq, Eq, Debug)]
pub enum Projection {
    /// Standard digital cinema (DCI).
    Dci { dci: Dci },
    /// IMAX.
    Imax { imax: Imax },
    /// Film.
    Film { film: Film },
}

/// Standard digital cinema (DCI) projector.
#[derive(uniffi::Enum, Clone, PartialEq, Eq, Debug)]
pub enum Dci {
    /// 2K resolution, xenon lamp.
    Lamp2K,
    /// 2K resolution, laser.
    Laser2K,
    /// 4K resolution, xenon lamp.
    Lamp4K,
    /// 4K resolution, laser.
    Laser4K,
    /// A DCI projection not listed above.
    Custom { id: String },
}

/// IMAX projection technology.
#[derive(uniffi::Enum, Clone, PartialEq, Eq, Debug)]
pub enum Imax {
    /// 2K, xenon lamp.
    Xenon2K,
    /// 4K, single laser: smaller screen, lower aspect ratio.
    Laser4KSingle,
    /// 4K, dual laser: larger screen, higher aspect ratio.
    Laser4KDual,
    /// 70mm film (15-perf).
    Film70mm,
    /// An IMAX projection not listed above.
    Custom { id: String },
}

/// Film projection technology.
#[derive(uniffi::Enum, Clone, PartialEq, Eq, Debug)]
pub enum Film {
    /// 35mm film.
    Mm35,
    /// 70mm film, non-IMAX.
    Mm70,
    /// A film format not listed above.
    Custom { id: String },
}

/// Theatrical auditorium class.
#[derive(uniffi::Enum, Clone, PartialEq, Eq, Debug)]
pub enum ScreenClass {
    /// A normal auditorium.
    Regular,
    /// Premium large format.
    Plf,
    /// VIP-style auditorium: reclining seats, food & beverage service.
    Vip,
    /// 4DX auditorium: motion seats plus environmental effects.
    FourDx,
    /// `ScreenX` auditorium: 3D presentation format.
    ScreenX,
    /// IMAX classic (1.43:1 aspect ratio).
    ImaxClassic,
    /// IMAX retrofit (1.90:1 aspect ratio).
    ImaxRetrofit,
    /// Ultra 4DX: `ScreenX` plus 4DX environmental effects.
    Ultra4Dx,
    /// An auditorium class not listed above.
    Custom { id: String },
}

/// Home release medium.
#[derive(uniffi::Enum, Clone, PartialEq, Eq, Debug)]
pub enum HomeMedium {
    /// DVD.
    Dvd,
    /// HD DVD.
    HdDvd,
    /// Blu-ray.
    BluRay,
    /// 4K Ultra HD Blu-ray.
    UhdBluRay,
    /// `LaserDisc`.
    LaserDisc,
    /// Streaming.
    Streaming,
    /// Linear television broadcast.
    Broadcast,
    /// Telecine: theatrical-quality video transfer.
    Telecine,
    /// A medium not listed above.
    Custom { id: String },
}

/// A film's board certifications, by supported board.
#[derive(uniffi::Record, Clone, PartialEq, Eq, Debug)]
pub struct Certifications {
    /// The US (MPAA) certification, e.g. `"R"`.
    pub us: Option<String>,
    /// The UK (BBFC) certification, e.g. `"15"`.
    pub gb: Option<String>,
    /// The German (FSK) certification, e.g. `"FSK 16"`.
    pub de: Option<String>,
}

/// User preferences, persisted with the log.
#[derive(uniffi::Record, Clone, PartialEq, Eq, Debug)]
pub struct Settings {
    /// The board whose certification the list view shows.
    pub selected_board: RatingBoard,
}

/// A TMDB genre.
#[derive(uniffi::Record, Clone, PartialEq, Eq, Debug)]
pub struct Genre {
    /// The TMDB genre id.
    pub id: i32,
    /// The genre name.
    pub name: String,
}

/// A full movie record (TMDB movie detail / find result).
#[derive(uniffi::Record, Clone, PartialEq, Debug)]
pub struct Movie {
    /// The TMDB numeric id.
    pub id: i64,
    /// The title.
    pub title: String,
    /// The original title.
    pub original_title: String,
    /// The original language (ISO 639-1).
    pub original_language: String,
    /// The release date (`YYYY-MM-DD`), if known.
    pub release_date: Option<String>,
    /// The runtime in minutes, if known.
    pub runtime: Option<i32>,
    /// The genres.
    pub genres: Vec<Genre>,
    /// A short overview.
    pub overview: String,
    /// The tagline, if known.
    pub tagline: Option<String>,
    /// The production status.
    pub status: String,
    /// The poster path, if known.
    pub poster_path: Option<String>,
    /// The backdrop path, if known.
    pub backdrop_path: Option<String>,
    /// The average vote.
    pub vote_average: f64,
    /// The vote count.
    pub vote_count: i64,
    /// The IMDB id, if known.
    pub imdb_id: Option<String>,
    /// The popularity score.
    pub popularity: f64,
    /// Whether the film is marked adult.
    pub adult: bool,
    /// The board certifications, if fetched.
    pub certifications: Option<Certifications>,
}

/// A lightweight movie record (search result).
#[derive(uniffi::Record, Clone, PartialEq, Debug)]
pub struct MovieSummary {
    /// The TMDB numeric id.
    pub id: i64,
    /// The title.
    pub title: String,
    /// The original title.
    pub original_title: String,
    /// The original language.
    pub original_language: String,
    /// The release date, if known.
    pub release_date: Option<String>,
    /// A short overview.
    pub overview: String,
    /// The TMDB genre ids.
    pub genre_ids: Vec<i32>,
    /// The poster path, if known.
    pub poster_path: Option<String>,
    /// The backdrop path, if known.
    pub backdrop_path: Option<String>,
    /// The average vote.
    pub vote_average: f64,
    /// The vote count.
    pub vote_count: i64,
    /// The popularity score.
    pub popularity: f64,
    /// Whether the film is marked adult.
    pub adult: bool,
}

/// A page of search results.
#[derive(uniffi::Record, Clone, PartialEq, Debug)]
pub struct SearchPage {
    /// The current page number.
    pub page: i32,
    /// The results on this page.
    pub results: Vec<MovieSummary>,
    /// The total number of pages.
    pub total_pages: i32,
    /// The total number of results.
    pub total_results: i32,
}

/// One viewing of a film.
#[derive(uniffi::Record, Clone, PartialEq, Debug)]
pub struct WatchLog {
    /// Row id; `0` until persisted by the store.
    pub id: i64,
    /// Canonical identifier of the film.
    pub film_id: FilmId,
    /// Denormalized IMDB id, if known.
    pub imdb_id: Option<String>,
    /// Denormalized TMDB id, if known.
    pub tmdb_id: Option<i64>,
    /// Denormalized film title.
    pub title: String,
    /// Denormalized release year, if known.
    pub release_year: Option<i32>,
    /// Denormalized TMDB poster path, if known.
    pub poster_path: Option<String>,
    /// Denormalized board certifications, if fetched.
    pub certifications: Option<Certifications>,
    /// Where it was watched.
    pub venue: Venue,
    /// The instant the film was watched, in UTC (RFC 3339).
    pub watched_at: String,
    /// Free-form notes.
    pub notes: Option<String>,
    /// Your thumbs-up/thumbs-down rating, if given.
    pub rating: Option<Rating>,
    /// Whether this is a rewatch of a film logged previously.
    pub rewatch: bool,
    /// How many times this film has been logged, including this viewing.
    pub watch_count: Option<i32>,
    /// How it was viewed, if known.
    pub format: Option<Format>,
}

/// Aggregated statistics over all stored watch logs.
#[derive(uniffi::Record, Clone, PartialEq, Eq, Debug)]
pub struct Stats {
    /// Total viewings (every watch log).
    pub total_viewings: u32,
    /// Number of distinct films.
    pub distinct_films: u32,
    /// Viewings by venue.
    pub viewings_by_venue: VenueCounts,
    /// Viewings by UTC calendar year, ascending.
    pub viewings_by_year: Vec<YearCount>,
    /// Viewings flagged as a rewatch.
    pub rewatches: u32,
    /// Top films by viewing count, descending (title as tie-break).
    pub top_films: Vec<FilmViewings>,
}

/// Viewings broken down by venue.
#[derive(uniffi::Record, Clone, Copy, PartialEq, Eq, Debug)]
pub struct VenueCounts {
    /// Theatrical screenings.
    pub theatre: u32,
    /// Home viewings.
    pub home: u32,
}

/// Viewings for one UTC calendar year.
#[derive(uniffi::Record, Clone, Copy, PartialEq, Eq, Debug)]
pub struct YearCount {
    /// The UTC calendar year.
    pub year: i32,
    /// Viewings in that year.
    pub count: u32,
}

/// A single film's viewing count, for ranked reports.
#[derive(uniffi::Record, Clone, PartialEq, Eq, Debug)]
pub struct FilmViewings {
    /// Denormalized film title.
    pub title: String,
    /// Number of times this film has been watched.
    pub count: u32,
}

/// An error from the logger.
#[derive(uniffi::Error, Clone, Debug)]
pub enum LoggerError {
    /// An I/O failure.
    Io { detail: String },
    /// A database failure.
    Database { detail: String },
    /// A JSON (de)serialization failure.
    Serde { detail: String },
    /// An HTTP transport failure.
    Http { detail: String },
    /// The TMDB API returned an error status.
    Api { status: i32, detail: String },
    /// The requested film was not found.
    FilmNotFound,
    /// A watch log id was not found in the store.
    LogNotFound { id: i64 },
    /// The database schema is newer than this build understands.
    SchemaTooNew { version: i64 },
    /// No TMDB API key was baked in at compile time.
    NoCompiledApiKey,
    /// A viewing format was attached to a log whose venue it does not match.
    VenueFormatMismatch { venue: Venue, format: String },
    /// A theatre format's projection cannot be screened in its auditorium class.
    ProjectionScreenMismatch { detail: String },
    /// A theatre format's presentation flag is incompatible.
    FlagMismatch { detail: String },
    /// A watch log's `watch_count` is below one.
    InvalidWatchCount { count: i32 },
}

impl std::fmt::Display for LoggerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io { detail }
            | Self::Database { detail }
            | Self::Serde { detail }
            | Self::Http { detail }
            | Self::ProjectionScreenMismatch { detail }
            | Self::FlagMismatch { detail } => write!(f, "{detail}"),
            Self::Api { status, detail } => {
                write!(f, "TMDB API error (HTTP {status}): {detail}")
            }
            Self::FilmNotFound => write!(f, "film not found"),
            Self::LogNotFound { id } => write!(f, "log {id} was not found"),
            Self::SchemaTooNew { version } => {
                write!(f, "database schema {version} is newer than this build")
            }
            Self::NoCompiledApiKey => write!(f, "no TMDB API key was compiled in"),
            Self::VenueFormatMismatch { format, .. } => {
                write!(f, "viewing format {format} does not match the venue")
            }
            Self::InvalidWatchCount { count } => {
                write!(f, "watch count {count} is less than 1")
            }
        }
    }
}
