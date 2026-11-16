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

/// One rating board TMDB reports certifications for.
#[derive(uniffi::Record, Clone, PartialEq, Eq, Debug)]
pub struct RatingBoard {
    /// The TMDB region key, e.g. `"US"`.
    pub region: String,
    /// The board's display name.
    pub name: String,
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

/// One region's certification.
#[derive(uniffi::Record, Clone, PartialEq, Eq, Debug)]
pub struct Certification {
    /// The TMDB region key, e.g. `"US"`.
    pub region: String,
    /// The board's certification, e.g. `"R"`.
    pub rating: String,
}

/// A film's board certifications.
#[derive(uniffi::Record, Clone, PartialEq, Eq, Debug)]
pub struct Certifications {
    /// Every known region certification, ordered by region.
    pub values: Vec<Certification>,
}

/// User preferences, persisted with the log.
#[derive(uniffi::Record, Clone, PartialEq, Eq, Debug)]
pub struct Settings {
    /// The region code of the board the list view shows.
    pub selected_board: String,
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
    /// The top-billed cast.
    pub cast: Vec<CastMember>,
    /// The collection the film belongs to, if any.
    pub collection: Option<CollectionRef>,
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
    /// The IANA time zone the viewing time was entered in, if known.
    pub timezone: Option<String>,
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
    /// One-line display summary of `format`, maintained by the store.
    pub format_summary: String,
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
    /// Viewings by UTC calendar month (`YYYY-MM`), ascending.
    pub viewings_by_month: Vec<CountEntry>,
    /// Viewings by projection family: `dci`, `imax`, `film`, or `home`.
    pub viewings_by_projection_family: Vec<CountEntry>,
    /// Viewings by auditorium class.
    pub viewings_by_screen: Vec<CountEntry>,
    /// Home viewings by release medium.
    pub viewings_by_medium: Vec<CountEntry>,
    /// Viewings per presentation flag: `three_d`, `hfr`, or `dbox`.
    pub viewings_by_flag: Vec<CountEntry>,
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

/// One credited cast member of a film.
#[derive(uniffi::Record, Clone, PartialEq, Eq, Debug)]
pub struct CastMember {
    /// The TMDB person id.
    pub id: i64,
    /// The actor's name.
    pub name: String,
    /// The character played, if known.
    pub character: Option<String>,
    /// The billing order (0 is top-billed).
    pub order: i32,
}

/// A reference to the collection (franchise) a film belongs to.
#[derive(uniffi::Record, Clone, PartialEq, Eq, Debug)]
pub struct CollectionRef {
    /// The TMDB collection id.
    pub id: i64,
    /// The collection name.
    pub name: String,
    /// The collection poster path, if known.
    pub poster_path: Option<String>,
}

/// A film's stored metadata plus the user's own film-level state.
#[derive(uniffi::Record, Clone, PartialEq, Debug)]
pub struct FilmMetadata {
    /// Canonical film identifier.
    pub film_id: FilmId,
    /// The TMDB numeric id, if known.
    pub tmdb_id: Option<i64>,
    /// The IMDB id, if known.
    pub imdb_id: Option<String>,
    /// The title.
    pub title: String,
    /// The original title.
    pub original_title: Option<String>,
    /// The release date (`YYYY-MM-DD`), if known.
    pub release_date: Option<String>,
    /// The release year, if known.
    pub release_year: Option<i32>,
    /// The runtime in minutes, if known.
    pub runtime_minutes: Option<i32>,
    /// A short overview.
    pub overview: Option<String>,
    /// The tagline, if known.
    pub tagline: Option<String>,
    /// The original language (ISO 639-1), if known.
    pub original_language: Option<String>,
    /// The poster path, if known.
    pub poster_path: Option<String>,
    /// The backdrop path, if known.
    pub backdrop_path: Option<String>,
    /// The average vote, if known.
    pub vote_average: Option<f64>,
    /// The vote count, if known.
    pub vote_count: Option<i64>,
    /// The popularity score, if known.
    pub popularity: Option<f64>,
    /// Whether the film is marked adult.
    pub adult: bool,
    /// The genres.
    pub genres: Vec<Genre>,
    /// The top-billed cast.
    pub cast: Vec<CastMember>,
    /// The board certifications, if fetched.
    pub certifications: Option<Certifications>,
    /// The collection the film belongs to, if any.
    pub collection: Option<CollectionRef>,
    /// Whether the user marked the film a favorite.
    pub favorite: bool,
    /// Whether the user put the film on their watchlist.
    pub watchlisted: bool,
    /// When the metadata was last refreshed (RFC 3339).
    pub metadata_refreshed_at: Option<String>,
}

/// One film in the library list, with its viewing summary.
#[derive(uniffi::Record, Clone, PartialEq, Debug)]
pub struct FilmListEntry {
    /// The film metadata.
    pub film: FilmMetadata,
    /// Number of stored viewings.
    pub viewings: u32,
    /// The most recent viewing instant (RFC 3339), if any.
    pub last_watched_at: Option<String>,
}

/// The viewing statistics for a single film.
#[derive(uniffi::Record, Clone, PartialEq, Eq, Debug)]
pub struct FilmStats {
    /// Number of viewings.
    pub viewings: u32,
    /// Theatrical viewings.
    pub theatre: u32,
    /// Home viewings.
    pub home: u32,
    /// The first viewing instant (RFC 3339), if any.
    pub first_watched_at: Option<String>,
    /// The most recent viewing instant (RFC 3339), if any.
    pub last_watched_at: Option<String>,
    /// Viewings per format summary.
    pub formats: Vec<CountEntry>,
}

/// A film's page: metadata, viewing history, and statistics.
#[derive(uniffi::Record, Clone, PartialEq, Debug)]
pub struct FilmPage {
    /// The film metadata.
    pub film: FilmMetadata,
    /// The film's viewings, most recent first.
    pub logs: Vec<WatchLog>,
    /// Viewing statistics.
    pub stats: FilmStats,
}

/// The user-supplied details of a new viewing.
#[derive(uniffi::Record, Clone, PartialEq, Eq, Debug, Default)]
pub struct NewLogDetails {
    /// When the film was watched (RFC 3339); defaults to now.
    pub watched_at: Option<String>,
    /// The IANA time zone the viewing time was entered in, if known.
    pub timezone: Option<String>,
    /// Free-form notes.
    pub notes: Option<String>,
    /// A thumbs-up/thumbs-down rating.
    pub rating: Option<Rating>,
}

/// A `(key, count)` pair, for a breakdown map.
#[derive(uniffi::Record, Clone, PartialEq, Eq, Debug)]
pub struct CountEntry {
    /// The entry's key.
    pub key: String,
    /// The entry's count.
    pub count: u32,
}

/// A selectable catalog entry: a stable key and its display label.
#[derive(uniffi::Record, Clone, PartialEq, Eq, Debug)]
pub struct FormatOption {
    /// The key used in a serialized [`Format`].
    pub key: String,
    /// The human-readable label.
    pub label: String,
}

/// A named group of catalog options (a family's technologies or screens).
#[derive(uniffi::Record, Clone, PartialEq, Eq, Debug)]
pub struct FormatGroup {
    /// The group key (the family key).
    pub key: String,
    /// The group's options.
    pub options: Vec<FormatOption>,
}

/// One legal projection × technology × auditorium combination.
#[derive(uniffi::Record, Clone, PartialEq, Eq, Debug)]
pub struct FormatLegality {
    /// The projection family: `dci`, `imax`, or `film`.
    pub family: String,
    /// The technology key.
    pub technology: String,
    /// The auditorium key.
    pub screen: String,
    /// The flags this combination may carry.
    pub allowed_flags: Vec<String>,
    /// The flags this combination must carry.
    pub forced_flags: Vec<String>,
}

/// The viewing-format catalog both apps render.
#[derive(uniffi::Record, Clone, PartialEq, Eq, Debug)]
pub struct FormatCatalog {
    /// The projection families, in display order.
    pub families: Vec<FormatOption>,
    /// The technologies per family.
    pub technologies: Vec<FormatGroup>,
    /// The auditoria legal for each family.
    pub screens: Vec<FormatGroup>,
    /// The home release mediums.
    pub mediums: Vec<FormatOption>,
    /// The presentation flags.
    pub flag_labels: Vec<FormatOption>,
    /// Every legal projection × technology × auditorium combination.
    pub legal: Vec<FormatLegality>,
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
    /// A viewing format is required on the log.
    MissingFormat { venue: Venue },
    /// A viewing format was attached to a log whose venue it does not match.
    VenueFormatMismatch { venue: Venue, format: String },
    /// A theatre format's projection cannot be screened in its auditorium class.
    ProjectionScreenMismatch { detail: String },
    /// A theatre format's presentation flag is incompatible.
    FlagMismatch { detail: String },
    /// A watch log's `watch_count` is below one.
    InvalidWatchCount { count: i32 },
    /// An import file was rejected.
    InvalidImport { detail: String },
    /// The film has no TMDB id, so it cannot be refreshed.
    NoTmdbId { film_id: FilmId },
    /// A film id string could not be parsed.
    InvalidFilmId { value: String },
    /// A stored log document is corrupt or unreadable.
    InvalidStoredLog { id: i64 },
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
            Self::MissingFormat { venue } => {
                write!(f, "a viewing format is required for a {venue:?} log")
            }
            Self::VenueFormatMismatch { format, .. } => {
                write!(f, "viewing format {format} does not match the venue")
            }
            Self::InvalidWatchCount { count } => {
                write!(f, "watch count {count} is less than 1")
            }
            Self::InvalidImport { detail } => write!(f, "import rejected: {detail}"),
            Self::NoTmdbId { film_id } => write!(f, "film {film_id:?} has no TMDB id"),
            Self::InvalidFilmId { value } => write!(f, "invalid film id: {value}"),
            Self::InvalidStoredLog { id } => write!(f, "stored log {id} is corrupt"),
        }
    }
}
