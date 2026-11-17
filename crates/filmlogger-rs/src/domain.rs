//! Domain types for the watch log.
//!
//! These are the stable, serializable core of the library: a [`WatchLog`]
//! records one viewing of a [`FilmId`], in a [`Venue`], at a UTC instant,
//! with an optional viewing [`Format`]. Films are identified by IMDB ID when
//! known, falling back to the TMDB numeric ID.

use std::collections::BTreeMap;

use chrono::{DateTime, Local, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::error::Error;
use crate::tmdb::Movie;

/// Where a film was watched.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Venue {
    /// A theatrical screening.
    Theatre,
    /// A home viewing.
    Home,
}

/// Canonical film identifier.
///
/// Serialized as a bare IMDB id string (`"tt0137523"`) or, for a film with no
/// IMDB id, a TMDB-prefixed string (`"tmdb:550"`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FilmId {
    /// An IMDB id, e.g. `"tt0137523"`.
    Imdb(String),
    /// A TMDB numeric id, used when the film has no IMDB id.
    Tmdb(i64),
}

impl FilmId {
    /// The canonical storage/display key: `"tt0137523"` or `"tmdb:550"`.
    #[must_use]
    pub fn key(&self) -> String {
        match self {
            FilmId::Imdb(s) => s.clone(),
            FilmId::Tmdb(n) => format!("tmdb:{n}"),
        }
    }

    /// Parse a key produced by [`Self::key`].
    ///
    /// Only the `tmdb:` prefix is strict: `tmdb:abc` is invalid. Every other
    /// string is an IMDB id.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidFilmId`] for a malformed `tmdb:` key.
    pub fn from_key(key: &str) -> Result<Self, Error> {
        if let Some(n) = key.strip_prefix("tmdb:") {
            let parsed = n.parse::<i64>().map_err(|_| Error::InvalidFilmId {
                value: key.to_owned(),
            })?;
            Ok(FilmId::Tmdb(parsed))
        } else {
            Ok(FilmId::Imdb(key.to_owned()))
        }
    }
}

impl Serialize for FilmId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.key())
    }
}

impl<'de> Deserialize<'de> for FilmId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        FilmId::from_key(&s).map_err(serde::de::Error::custom)
    }
}

/// How a film was viewed. Venue applicability is structural: a
/// [`Format::Theatre`] is a projection technology × auditorium class; a
/// [`Format::Home`] is a release medium. A theatre format can never be
/// attached to a home log and vice versa.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Format {
    /// A theatrical screening.
    #[serde(rename = "theatre")]
    Theatre {
        /// The projection technology.
        projection: Projection,
        /// The auditorium class.
        screen: ScreenClass,
        /// Presented in 3D.
        #[serde(default)]
        three_d: bool,
        /// Presented at high frame rate.
        #[serde(default)]
        hfr: bool,
        /// Presented on D-BOX motion seats.
        #[serde(default)]
        dbox: bool,
    },
    /// A home viewing.
    #[serde(rename = "home")]
    Home {
        /// The release medium.
        medium: HomeMedium,
        /// Presented in 3D.
        #[serde(default)]
        three_d: bool,
        /// Presented at high frame rate.
        #[serde(default)]
        hfr: bool,
    },
}

impl Format {
    /// The venue this format applies to.
    ///
    /// [`Format::Theatre`] → [`Venue::Theatre`]; [`Format::Home`] →
    /// [`Venue::Home`].
    #[must_use]
    pub fn venue(&self) -> Venue {
        match self {
            Format::Theatre { .. } => Venue::Theatre,
            Format::Home { .. } => Venue::Home,
        }
    }

    /// Validate a theatre format against projection/screen compatibility and
    /// presentation-flag policies.
    ///
    /// [`Format::Theatre`] checks that the projection runs in the screen and
    /// that every flag is allowed (and, where the screen forces one, on) for
    /// both the projection and the screen. [`Format::Home`] checks the
    /// medium-specific flag rules (3D Blu-ray forces 3D; UHD Blu-ray allows
    /// optional HFR; other media carry neither).
    ///
    /// # Errors
    ///
    /// Returns [`Error::ProjectionScreenMismatch`] if the projection does not
    /// run in the screen, [`Error::FlagMismatch`] if a theatre flag is
    /// incompatible, or [`Error::HomeFlagMismatch`] if a home flag is
    /// incompatible.
    #[must_use = "a viewing format must be validated before use"]
    pub fn validate(&self) -> Result<(), Error> {
        if let Format::Home {
            medium,
            three_d,
            hfr,
        } = self
        {
            for (flag, on) in [(Flag::ThreeD, *three_d), (Flag::Hfr, *hfr)] {
                let allowed = medium.allows_flag(flag);
                let forced = medium.forces_flag(flag);
                let ok = if forced {
                    on && allowed
                } else {
                    !on || allowed
                };
                if !ok {
                    return Err(Error::HomeFlagMismatch {
                        medium: medium.clone(),
                        flag,
                    });
                }
            }
            return Ok(());
        }
        let Format::Theatre {
            projection,
            screen,
            three_d,
            hfr,
            dbox,
        } = self
        else {
            unreachable!()
        };
        if !projection.runs_on(screen) {
            return Err(Error::ProjectionScreenMismatch {
                projection: projection.clone(),
                screen: screen.clone(),
            });
        }
        for (flag, on) in [
            (Flag::ThreeD, *three_d),
            (Flag::Hfr, *hfr),
            (Flag::Dbox, *dbox),
        ] {
            let proj_ok = projection.allows(flag);
            let screen_ok = screen.allows(flag);
            let forced = screen.forces(flag);
            let ok = if forced {
                on && proj_ok && screen_ok
            } else {
                !on || (proj_ok && screen_ok)
            };
            if !ok {
                return Err(Error::FlagMismatch {
                    flag,
                    projection: projection.clone(),
                    screen: screen.clone(),
                });
            }
        }
        Ok(())
    }
}

/// Theatrical projection technology, grouped by projection family.
///
/// The family mirrors how a technology is branded and screened: standard
/// digital cinema ([`Dci`]), [`Imax`], and [`Film`]. Each family owns its
/// resolution and light-source ladder; a brand-new technology is logged through
/// the family's `Custom` escape hatch without a library release.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Projection {
    /// Standard digital cinema (DCI).
    #[serde(rename = "dci")]
    Dci(Dci),
    /// IMAX.
    #[serde(rename = "imax")]
    Imax(Imax),
    /// Film.
    #[serde(rename = "film")]
    Film(Film),
}

/// Standard digital cinema (DCI) projector.
///
/// Light source defaults to xenon lamp unless a laser variant is explicit
/// ("DCI is without laser unless specified").
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Dci {
    /// 2K resolution, xenon lamp.
    #[serde(rename = "2k_lamp")]
    Lamp2K,
    /// 2K resolution, laser.
    #[serde(rename = "2k_laser")]
    Laser2K,
    /// 4K resolution, xenon lamp.
    #[serde(rename = "4k_lamp")]
    Lamp4K,
    /// 4K resolution, laser.
    #[serde(rename = "4k_laser")]
    Laser4K,
    /// A DCI projection not listed above.
    #[serde(rename = "custom")]
    Custom(String),
}

/// IMAX projection technology.
///
/// The 4K laser variants are distinct: single laser drives a smaller screen at a
/// lower aspect ratio, dual laser a larger screen at a higher one.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Imax {
    /// 2K, xenon lamp (the "IMAX with xenon" digital house).
    #[serde(rename = "2k_xenon")]
    Xenon2K,
    /// 4K, single laser: smaller screen, lower aspect ratio.
    #[serde(rename = "4k_single_laser")]
    Laser4KSingle,
    /// 4K, dual laser: larger screen, higher aspect ratio.
    #[serde(rename = "4k_dual_laser")]
    Laser4KDual,
    /// 70mm film (15-perf).
    #[serde(rename = "70mm")]
    Film70mm,
    /// An IMAX projection not listed above.
    #[serde(rename = "custom")]
    Custom(String),
}

/// Film projection technology.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Film {
    /// 35mm film.
    #[serde(rename = "35mm")]
    Mm35,
    /// 70mm film, non-IMAX (e.g. Todd-AO).
    #[serde(rename = "70mm")]
    Mm70,
    /// A film format not listed above.
    #[serde(rename = "custom")]
    Custom(String),
}

/// Theatrical auditorium class.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ScreenClass {
    /// A normal auditorium.
    #[serde(rename = "regular")]
    Regular,
    /// Premium large format.
    #[serde(rename = "plf")]
    Plf,
    /// VIP-style auditorium: reclining seats, food & beverage service.
    #[serde(rename = "vip")]
    Vip,
    /// 4DX auditorium: motion seats plus environmental effects.
    #[serde(rename = "4dx")]
    FourDx,
    /// `ScreenX` auditorium: 3D presentation format.
    #[serde(rename = "screenx")]
    ScreenX,
    /// IMAX classic (1.43:1 aspect ratio).
    #[serde(rename = "imax_class")]
    ImaxClassic,
    /// IMAX retrofit (1.90:1 aspect ratio).
    #[serde(rename = "imax_retrofit")]
    ImaxRetrofit,
    /// Ultra 4DX: [`ScreenClass::ScreenX`] plus 4DX environmental effects.
    #[serde(rename = "ultra_4dx")]
    Ultra4Dx,
    /// An auditorium class not listed above.
    #[serde(rename = "custom")]
    Custom(String),
}

/// A presentation flag for a theatrical screening.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Flag {
    /// Presented in 3D.
    ThreeD,
    /// Presented at high frame rate.
    Hfr,
    /// Presented on D-BOX motion seats.
    Dbox,
}

impl HomeMedium {
    /// Whether this medium can carry the given presentation flag.
    #[must_use]
    pub fn allows_flag(&self, flag: Flag) -> bool {
        match self {
            HomeMedium::BluRay3D => matches!(flag, Flag::ThreeD),
            HomeMedium::UhdBluRay => matches!(flag, Flag::Hfr),
            _ => false,
        }
    }

    /// Whether this medium always carries the given presentation flag.
    #[must_use]
    pub fn forces_flag(&self, flag: Flag) -> bool {
        matches!((self, flag), (HomeMedium::BluRay3D, Flag::ThreeD))
    }
}

impl Projection {
    /// Whether this projection can present the given presentation flag.
    #[must_use]
    pub fn allows(&self, flag: Flag) -> bool {
        match self {
            Projection::Dci(_) => true,
            Projection::Imax(Imax::Film70mm) => false,
            Projection::Imax(_) => !matches!(flag, Flag::Dbox),
            Projection::Film(_) => matches!(flag, Flag::ThreeD),
        }
    }

    /// Whether this projection can be screened in the given auditorium class.
    #[must_use]
    pub fn runs_on(&self, screen: &ScreenClass) -> bool {
        match self {
            Projection::Imax(_) => {
                matches!(screen, ScreenClass::ImaxClassic | ScreenClass::ImaxRetrofit)
            }
            Projection::Dci(_) => {
                !matches!(screen, ScreenClass::ImaxClassic | ScreenClass::ImaxRetrofit)
            }
            Projection::Film(_) => matches!(screen, ScreenClass::Regular),
        }
    }
}

impl ScreenClass {
    /// Whether the given presentation flag may be on in this auditorium.
    #[must_use]
    pub fn allows(&self, flag: Flag) -> bool {
        match self {
            ScreenClass::Regular | ScreenClass::Plf | ScreenClass::Custom(_) => true,
            ScreenClass::Vip
            | ScreenClass::FourDx
            | ScreenClass::Ultra4Dx
            | ScreenClass::ImaxClassic
            | ScreenClass::ImaxRetrofit => !matches!(flag, Flag::Dbox),
            ScreenClass::ScreenX => false,
        }
    }

    /// Whether the given presentation flag must be on in this auditorium.
    #[must_use]
    pub fn forces(&self, flag: Flag) -> bool {
        matches!(flag, Flag::ThreeD) && matches!(self, ScreenClass::FourDx | ScreenClass::Ultra4Dx)
    }

    /// The screen's aspect ratio, when it distinguishes the class.
    ///
    /// [`ScreenClass::ImaxClassic`] is `1.43:1`; [`ScreenClass::ImaxRetrofit`]
    /// is `1.90:1`; other classes have no single canonical ratio.
    #[must_use]
    pub fn aspect_ratio(&self) -> Option<&'static str> {
        match self {
            ScreenClass::ImaxClassic => Some("1.43:1"),
            ScreenClass::ImaxRetrofit => Some("1.90:1"),
            _ => None,
        }
    }
}

/// Home release medium.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HomeMedium {
    /// DVD.
    #[serde(rename = "dvd")]
    Dvd,
    /// HD DVD.
    #[serde(rename = "hd_dvd")]
    HdDvd,
    /// Blu-ray.
    #[serde(rename = "bluray")]
    BluRay,
    /// 3D Blu-ray.
    #[serde(rename = "bluray_3d")]
    BluRay3D,
    /// 4K Ultra HD Blu-ray.
    #[serde(rename = "uhd_bluray")]
    UhdBluRay,
    /// `LaserDisc`.
    #[serde(rename = "laserdisc")]
    LaserDisc,
    /// Streaming.
    #[serde(rename = "streaming")]
    Streaming,
    /// Linear television broadcast.
    #[serde(rename = "broadcast")]
    Broadcast,
    /// Telecine: theatrical-quality video transfer.
    #[serde(rename = "telecine")]
    Telecine,
    /// A medium not listed above.
    #[serde(rename = "custom")]
    Custom(String),
}

/// A thumbs-up or thumbs-down rating of a viewing.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Rating {
    /// Thumbs up.
    Up,
    /// Thumbs down.
    Down,
}

/// One rating board TMDB reports certifications for.
///
/// A board is keyed by the ISO 3166-1 region code TMDB uses (`"US"`,
/// `"GB"`, `"CA-QC"`), with a human-readable name. Distinct from [`Rating`]
/// (a thumbs up/down of a viewing): a board certifies a *film* (e.g. `R`,
/// `15`, `FSK 16`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RatingBoard {
    /// The TMDB region key, e.g. `"US"`.
    pub region: String,
    /// The board's display name, e.g. `"United States (MPA)"`.
    pub name: String,
}

impl RatingBoard {
    fn new(region: &str, name: &str) -> Self {
        Self {
            region: region.to_owned(),
            name: name.to_owned(),
        }
    }
}

/// Every movie certification board TMDB's `/certification/movie/list`
/// endpoint reports.
const RATING_BOARDS: &[(&str, &str)] = &[
    ("AR", "Argentina"),
    ("AT", "Austria"),
    ("AU", "Australia (ACB)"),
    ("BE", "Belgium"),
    ("BG", "Bulgaria"),
    ("BR", "Brazil (ClassInd)"),
    ("CA", "Canada"),
    ("CA-QC", "Canada (Quebec)"),
    ("CH", "Switzerland"),
    ("CL", "Chile"),
    ("CZ", "Czechia"),
    ("DE", "Germany (FSK)"),
    ("DK", "Denmark"),
    ("ES", "Spain"),
    ("FI", "Finland"),
    ("FR", "France (CNC)"),
    ("GB", "United Kingdom (BBFC)"),
    ("GR", "Greece"),
    ("HK", "Hong Kong"),
    ("HU", "Hungary"),
    ("ID", "Indonesia"),
    ("IE", "Ireland"),
    ("IL", "Israel"),
    ("IN", "India (CBFC)"),
    ("IT", "Italy"),
    ("JP", "Japan (Eirin)"),
    ("KR", "South Korea"),
    ("LT", "Lithuania"),
    ("LU", "Luxembourg"),
    ("LV", "Latvia"),
    ("MO", "Macau"),
    ("MX", "Mexico"),
    ("MY", "Malaysia"),
    ("NL", "Netherlands"),
    ("NO", "Norway"),
    ("NZ", "New Zealand"),
    ("PH", "Philippines"),
    ("PL", "Poland"),
    ("PR", "Puerto Rico"),
    ("PT", "Portugal"),
    ("RO", "Romania"),
    ("RU", "Russia"),
    ("SE", "Sweden"),
    ("SG", "Singapore"),
    ("SK", "Slovakia"),
    ("TH", "Thailand"),
    ("TR", "Turkey"),
    ("TW", "Taiwan"),
    ("UA", "Ukraine"),
    ("US", "United States (MPA)"),
    ("VI", "US Virgin Islands"),
    ("ZA", "South Africa"),
];

/// All movie certification boards TMDB supports, ordered by region code.
#[must_use]
pub fn rating_boards() -> Vec<RatingBoard> {
    RATING_BOARDS
        .iter()
        .map(|(region, name)| RatingBoard::new(region, name))
        .collect()
}

/// Look up a board by region code (case-insensitive).
#[must_use]
pub fn rating_board(region: &str) -> Option<RatingBoard> {
    let region = normalize_board_key(region);
    RATING_BOARDS
        .iter()
        .find(|(code, _)| code.eq_ignore_ascii_case(&region))
        .map(|(code, name)| RatingBoard::new(code, name))
}

/// A film's board certifications, keyed by TMDB region code.
///
/// Unknown regions are simply absent. The legacy `{ us, gb, de }` shape is
/// accepted when deserializing older exports and logs.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct Certifications(BTreeMap<String, String>);

impl Certifications {
    /// Build a certification map, normalizing keys to uppercase region codes
    /// and dropping blank ratings.
    #[must_use]
    pub fn new(values: impl IntoIterator<Item = (String, String)>) -> Self {
        let mut certifications = Self::default();
        for (region, rating) in values {
            certifications.insert(region, rating);
        }
        certifications
    }

    /// Set one region's certification.
    pub fn insert(&mut self, region: impl AsRef<str>, rating: impl AsRef<str>) {
        let region = normalize_board_key(region.as_ref());
        let rating = rating.as_ref().trim();
        if !region.is_empty() && !rating.is_empty() {
            self.0.insert(region, rating.to_owned());
        }
    }

    /// The certification for `region`, if known.
    #[must_use]
    pub fn get(&self, region: &str) -> Option<&str> {
        self.0.get(&normalize_board_key(region)).map(String::as_str)
    }

    /// Iterate `(region, rating)` pairs in region order.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.0
            .iter()
            .map(|(region, rating)| (region.as_str(), rating.as_str()))
    }

    /// True when no board has a certification.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<'de> Deserialize<'de> for Certifications {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw: BTreeMap<String, Option<String>> = BTreeMap::deserialize(deserializer)?;
        let mut certifications = Self::default();
        for (region, rating) in raw {
            if let Some(rating) = rating {
                certifications.insert(region, rating);
            }
        }
        Ok(certifications)
    }
}

/// Normalize a board/region key, including the three legacy enum names.
fn normalize_board_key(value: &str) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "mpa" => "US".to_owned(),
        "bbfc" => "GB".to_owned(),
        "fsk" => "DE".to_owned(),
        _ => value.trim().to_ascii_uppercase(),
    }
}

/// A genre associated with a film.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Genre {
    /// The TMDB genre id.
    pub id: i32,
    /// The genre name.
    pub name: String,
}

/// One credited cast member of a film.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CollectionRef {
    /// The TMDB collection id.
    pub id: i64,
    /// The collection name.
    pub name: String,
    /// The collection poster path, if known.
    pub poster_path: Option<String>,
}

/// One film in a TMDB collection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CollectionPart {
    /// The TMDB numeric id.
    pub tmdb_id: i64,
    /// The title.
    pub title: String,
    /// The release date (`YYYY-MM-DD`), if known.
    pub release_date: Option<String>,
    /// The release year, parsed from the release date.
    pub release_year: Option<i32>,
    /// The poster path, if known.
    pub poster_path: Option<String>,
}

/// A TMDB collection and its films.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Collection {
    /// The TMDB collection id.
    pub id: i64,
    /// The collection name.
    pub name: String,
    /// A short overview, if known.
    pub overview: Option<String>,
    /// The collection poster path, if known.
    pub poster_path: Option<String>,
    /// The collection backdrop path, if known.
    pub backdrop_path: Option<String>,
    /// The films in the collection.
    pub parts: Vec<CollectionPart>,
}

/// A film's stored metadata plus the user's own film-level state.
///
/// TMDB-sourced columns are refreshed by [`crate::FilmLogger::refresh_film`];
/// `favorite` and `watchlisted` are owned by the user and survive a refresh.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    /// When the TMDB-sourced metadata was last refreshed.
    pub metadata_refreshed_at: Option<DateTime<Utc>>,
}

impl FilmMetadata {
    /// Build the metadata for a resolved TMDB movie.
    #[must_use]
    pub fn from_movie(movie: &Movie) -> Self {
        let film_id = match &movie.imdb_id {
            Some(imdb) => FilmId::Imdb(imdb.clone()),
            None => FilmId::Tmdb(movie.id),
        };
        Self {
            film_id,
            tmdb_id: Some(movie.id),
            imdb_id: movie.imdb_id.clone(),
            title: movie.title.clone(),
            original_title: Some(movie.original_title.clone()),
            release_date: movie.release_date.clone(),
            release_year: movie.release_year(),
            runtime_minutes: movie.runtime,
            overview: Some(movie.overview.clone()),
            tagline: movie.tagline.clone(),
            original_language: Some(movie.original_language.clone()),
            poster_path: movie.poster_path.clone(),
            backdrop_path: movie.backdrop_path.clone(),
            vote_average: Some(movie.vote_average),
            vote_count: Some(movie.vote_count),
            popularity: Some(movie.popularity),
            adult: movie.adult,
            genres: movie.genres.clone(),
            cast: movie.cast.clone(),
            certifications: movie.certifications.clone(),
            collection: movie.collection.clone(),
            favorite: false,
            watchlisted: false,
            metadata_refreshed_at: Some(Utc::now()),
        }
    }

    /// A minimal metadata row for a legacy direct-store insert, built from a
    /// log's denormalized fields.
    #[must_use]
    pub fn from_log(log: &WatchLog) -> Self {
        Self {
            film_id: log.film_id.clone(),
            tmdb_id: log.tmdb_id,
            imdb_id: log.imdb_id.clone(),
            title: log.title.clone(),
            original_title: None,
            release_date: None,
            release_year: log.release_year,
            runtime_minutes: None,
            overview: None,
            tagline: None,
            original_language: None,
            poster_path: log.poster_path.clone(),
            backdrop_path: None,
            vote_average: None,
            vote_count: None,
            popularity: None,
            adult: false,
            genres: Vec::new(),
            cast: Vec::new(),
            certifications: log.certifications.clone(),
            collection: None,
            favorite: false,
            watchlisted: false,
            metadata_refreshed_at: None,
        }
    }
}

/// One film in the library list, with its viewing summary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilmListEntry {
    /// The film metadata.
    pub film: FilmMetadata,
    /// Number of stored viewings.
    pub viewings: usize,
    /// The most recent viewing instant, if any.
    pub last_watched_at: Option<DateTime<Utc>>,
}

/// The viewing statistics for a single film.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FilmStats {
    /// Number of viewings.
    pub viewings: usize,
    /// Theatrical viewings.
    pub theatre: usize,
    /// Home viewings.
    pub home: usize,
    /// The first viewing instant, if any.
    pub first_watched_at: Option<DateTime<Utc>>,
    /// The most recent viewing instant, if any.
    pub last_watched_at: Option<DateTime<Utc>>,
    /// Viewings per format summary.
    pub formats: BTreeMap<String, usize>,
}

/// A film's page: metadata, its viewing history, and its statistics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilmPage {
    /// The film metadata.
    pub film: FilmMetadata,
    /// The film's viewings, most recent first.
    pub logs: Vec<WatchLog>,
    /// Viewing statistics.
    pub stats: FilmStats,
}

/// The user-supplied details of a new viewing.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct NewLogDetails {
    /// When the film was watched; defaults to now.
    pub watched_at: Option<DateTime<Utc>>,
    /// The IANA time zone the time was entered in, if known.
    pub timezone: Option<String>,
    /// Free-form notes.
    pub notes: Option<String>,
    /// A thumbs-up/thumbs-down rating.
    pub rating: Option<Rating>,
}

/// User preferences, persisted with the log (in the same database).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// The region code of the board whose certification the list view shows.
    #[serde(deserialize_with = "deserialize_selected_board")]
    pub selected_board: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            selected_board: "US".to_owned(),
        }
    }
}

/// Accept both region codes and the legacy `mpa`/`bbfc`/`fsk` names.
fn deserialize_selected_board<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    Ok(normalize_board_key(&value))
}

/// One viewing of a film.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    #[serde(default)]
    pub poster_path: Option<String>,
    /// Denormalized board certifications, if fetched.
    #[serde(default)]
    pub certifications: Option<Certifications>,
    /// Where it was watched.
    pub venue: Venue,
    /// The instant the film was watched, in UTC.
    pub watched_at: DateTime<Utc>,
    /// The IANA time zone the viewing time was entered in, if known.
    #[serde(default)]
    pub timezone: Option<String>,
    /// Free-form notes.
    pub notes: Option<String>,
    /// Your thumbs-up/thumbs-down rating, if given.
    #[serde(default)]
    pub rating: Option<Rating>,
    /// Whether this is a rewatch of a film logged previously. A read-only
    /// projection maintained by the store: true when this viewing is not the
    /// film's first, in chronological order (`watched_at`, then row id).
    #[serde(default)]
    pub rewatch: bool,
    /// How many times this film has been logged, including this viewing.
    /// A read-only projection maintained by the store.
    #[serde(default)]
    pub watch_count: Option<i32>,
    /// How it was viewed. Required on validation: see [`Self::validate`].
    pub format: Option<Format>,
    /// One-line display summary of `format`, maintained by the store.
    #[serde(default)]
    pub format_summary: String,
}

impl WatchLog {
    /// Create a log. `local` is the device-local wall time (e.g.
    /// [`Local::now`]); it is translated to UTC using the device's current
    /// timezone offset.
    pub fn new(
        film_id: FilmId,
        title: impl Into<String>,
        venue: Venue,
        local: DateTime<Local>,
    ) -> Self {
        Self {
            id: 0,
            film_id,
            imdb_id: None,
            tmdb_id: None,
            title: title.into(),
            release_year: None,
            poster_path: None,
            certifications: None,
            venue,
            watched_at: local.with_timezone(&Utc),
            timezone: None,
            notes: None,
            rating: None,
            rewatch: false,
            watch_count: None,
            format: None,
            format_summary: String::new(),
        }
    }

    /// Convenience: [`Self::new`] with [`Local::now`].
    pub fn now(film_id: FilmId, title: impl Into<String>, venue: Venue) -> Self {
        Self::new(film_id, title, venue, Local::now())
    }

    /// Create a log from a resolved TMDB movie.
    ///
    /// `film_id` is derived from the movie: its IMDB id when present, else its
    /// TMDB id. Denormalized fields (`imdb_id`, `tmdb_id`, `release_year`,
    /// `title`, `poster_path`, `certifications`) are filled from the movie.
    #[must_use]
    pub fn from_movie(movie: &Movie, venue: Venue, local: DateTime<Local>) -> Self {
        let film_id = match &movie.imdb_id {
            Some(imdb) => FilmId::Imdb(imdb.clone()),
            None => FilmId::Tmdb(movie.id),
        };
        Self {
            id: 0,
            film_id,
            imdb_id: movie.imdb_id.clone(),
            tmdb_id: Some(movie.id),
            title: movie.title.clone(),
            release_year: movie.release_year(),
            poster_path: movie.poster_path.clone(),
            certifications: movie.certifications.clone(),
            venue,
            watched_at: local.with_timezone(&Utc),
            timezone: None,
            notes: None,
            rating: None,
            rewatch: false,
            watch_count: None,
            format: None,
            format_summary: String::new(),
        }
    }

    /// Convenience: [`Self::from_movie`] with [`Local::now`].
    #[must_use]
    pub fn from_movie_now(movie: &Movie, venue: Venue) -> Self {
        Self::from_movie(movie, venue, Local::now())
    }

    /// Create a log of a stored film with a known viewing format.
    ///
    /// The film's denormalized columns (`imdb_id`, `tmdb_id`, `title`,
    /// `release_year`, `poster_path`, `certifications`) are copied from
    /// `film`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::VenueFormatMismatch`] if `format.venue() != venue`, or
    /// a validation error if the format is internally inconsistent.
    pub fn from_film(
        film: &FilmMetadata,
        venue: Venue,
        format: Format,
        local: DateTime<Local>,
    ) -> Result<Self, Error> {
        let mut log = Self::new(film.film_id.clone(), &film.title, venue, local);
        log.imdb_id.clone_from(&film.imdb_id);
        log.tmdb_id = film.tmdb_id;
        log.release_year = film.release_year;
        log.poster_path.clone_from(&film.poster_path);
        log.certifications.clone_from(&film.certifications);
        log.attach_format(venue, format)?;
        Ok(log)
    }

    /// Set the derived display fields from the current `format`.
    pub fn set_derived(&mut self) {
        self.format_summary = self
            .format
            .as_ref()
            .map_or_else(String::new, Format::summary);
    }

    /// Validate `format` for `venue` and attach it.
    fn attach_format(&mut self, venue: Venue, format: Format) -> Result<(), Error> {
        let format = check_format(venue, format)?;
        self.format = Some(format);
        Ok(())
    }

    /// Create a log with a known viewing format.
    ///
    /// # Errors
    ///
    /// Returns [`Error::VenueFormatMismatch`] if `format.venue() != venue`, or a
    /// validation error if the format is internally inconsistent.
    pub fn with_format(
        film_id: FilmId,
        title: impl Into<String>,
        venue: Venue,
        format: Format,
        local: DateTime<Local>,
    ) -> Result<Self, Error> {
        let mut log = Self::new(film_id, title, venue, local);
        log.attach_format(venue, format)?;
        Ok(log)
    }

    /// Convenience: [`Self::with_format`] with [`Local::now`].
    ///
    /// # Errors
    ///
    /// As [`Self::with_format`].
    pub fn with_format_now(
        film_id: FilmId,
        title: impl Into<String>,
        venue: Venue,
        format: Format,
    ) -> Result<Self, Error> {
        Self::with_format(film_id, title, venue, format, Local::now())
    }

    /// Create a log from a resolved TMDB movie with a known viewing format.
    ///
    /// # Errors
    ///
    /// Returns [`Error::VenueFormatMismatch`] if `format.venue() != venue`, or a
    /// validation error if the format is internally inconsistent.
    pub fn from_movie_with_format(
        movie: &Movie,
        venue: Venue,
        format: Format,
        local: DateTime<Local>,
    ) -> Result<Self, Error> {
        let mut log = Self::from_movie(movie, venue, local);
        log.attach_format(venue, format)?;
        Ok(log)
    }

    /// Convenience: [`Self::from_movie_with_format`] with [`Local::now`].
    ///
    /// # Errors
    ///
    /// As [`Self::from_movie_with_format`].
    pub fn from_movie_with_format_now(
        movie: &Movie,
        venue: Venue,
        format: Format,
    ) -> Result<Self, Error> {
        Self::from_movie_with_format(movie, venue, format, Local::now())
    }

    /// Attach notes (builder-style).
    #[must_use]
    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }

    /// Attach a thumbs-up/thumbs-down rating (builder-style).
    #[must_use]
    pub fn with_rating(mut self, rating: Rating) -> Self {
        self.rating = Some(rating);
        self
    }

    /// Validate the log's invariants.
    ///
    /// Checks that [`Self::watch_count`], if present, is at least one, and
    /// that [`Self::format`] is present, matches the venue, and is internally
    /// consistent.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidWatchCount`] if `watch_count` is below one,
    /// [`Error::MissingFormat`] if no format is attached,
    /// [`Error::VenueFormatMismatch`] if the format does not match the venue,
    /// or a format validation error if the format is internally inconsistent.
    #[must_use = "a watch log must be validated before persistence"]
    pub fn validate(&self) -> Result<(), Error> {
        if let Some(n) = self.watch_count {
            if n < 1 {
                return Err(Error::InvalidWatchCount { count: n });
            }
        }
        let Some(format) = self.format.clone() else {
            return Err(Error::MissingFormat { venue: self.venue });
        };
        check_format(self.venue, format)?;
        Ok(())
    }
}

/// Validate a format against a venue and return it.
fn check_format(venue: Venue, format: Format) -> Result<Format, Error> {
    if format.venue() != venue {
        return Err(Error::VenueFormatMismatch { venue, format });
    }
    format.validate()?;
    Ok(format)
}

#[cfg(test)]
mod tests {
    use super::{
        Certifications, Dci, Film, FilmId, Flag, Format, HomeMedium, Imax, Projection, Rating,
        ScreenClass, Settings, Venue, WatchLog,
    };
    use crate::error::Error;
    use crate::tmdb::Movie;
    use chrono::{DateTime, Local, Utc};

    fn make_movie(imdb_id: Option<String>) -> Movie {
        Movie {
            id: 550,
            title: "Fight Club".into(),
            original_title: "Fight Club".into(),
            original_language: "en".into(),
            release_date: Some("1999-10-15".into()),
            runtime: Some(139),
            genres: vec![],
            overview: String::new(),
            tagline: None,
            status: "Released".into(),
            poster_path: None,
            backdrop_path: None,
            vote_average: 8.433,
            vote_count: 26279,
            imdb_id,
            popularity: 73.433,
            adult: false,
            certifications: None,
            cast: vec![],
            collection: None,
        }
    }

    #[test]
    fn film_id_serde_roundtrip() {
        assert_eq!(
            serde_json::to_string(&FilmId::Imdb("tt0137523".into())).unwrap(),
            r#""tt0137523""#
        );
        assert_eq!(
            serde_json::to_string(&FilmId::Tmdb(550)).unwrap(),
            r#""tmdb:550""#
        );
        let parsed: FilmId = serde_json::from_str(r#""tmdb:550""#).unwrap();
        assert_eq!(parsed, FilmId::Tmdb(550));
        let parsed: FilmId = serde_json::from_str(r#""tt123""#).unwrap();
        assert_eq!(parsed, FilmId::Imdb("tt123".into()));
    }

    #[test]
    fn venue_serde() {
        assert_eq!(
            serde_json::to_string(&Venue::Theatre).unwrap(),
            r#""theatre""#
        );
        assert_eq!(serde_json::to_string(&Venue::Home).unwrap(), r#""home""#);
        let v: Venue = serde_json::from_str(r#""theatre""#).unwrap();
        assert_eq!(v, Venue::Theatre);
        let v: Venue = serde_json::from_str(r#""home""#).unwrap();
        assert_eq!(v, Venue::Home);
    }

    #[test]
    fn watched_at_is_utc_instant() {
        let fixed = chrono::DateTime::parse_from_rfc3339("2026-10-25T18:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let local: DateTime<Local> = fixed.with_timezone(&Local);
        let log = WatchLog::new(
            FilmId::Imdb("tt0137523".into()),
            "Test",
            Venue::Theatre,
            local,
        );
        assert_eq!(log.watched_at, fixed);
    }

    #[test]
    fn from_movie_fills_fields() {
        let movie = make_movie(Some("tt0137523".into()));
        let log = WatchLog::from_movie(&movie, Venue::Theatre, Local::now());
        assert_eq!(log.film_id, FilmId::Imdb("tt0137523".into()));
        assert_eq!(log.imdb_id, Some("tt0137523".into()));
        assert_eq!(log.tmdb_id, Some(550));
        assert_eq!(log.release_year, Some(1999));
        assert_eq!(log.title, "Fight Club");

        let movie_no_imdb = make_movie(None);
        let log2 = WatchLog::from_movie(&movie_no_imdb, Venue::Home, Local::now());
        assert_eq!(log2.film_id, FilmId::Tmdb(550));
    }

    #[test]
    fn format_serde() {
        let dci = Format::Theatre {
            projection: Projection::Dci(Dci::Lamp4K),
            screen: ScreenClass::Plf,
            three_d: false,
            hfr: false,
            dbox: false,
        };
        assert_eq!(
            serde_json::to_string(&dci).unwrap(),
            r#"{"theatre":{"projection":{"dci":"4k_lamp"},"screen":"plf","three_d":false,"hfr":false,"dbox":false}}"#
        );

        let imax = Format::Theatre {
            projection: Projection::Imax(Imax::Laser4KDual),
            screen: ScreenClass::Regular,
            three_d: true,
            hfr: true,
            dbox: true,
        };
        assert_eq!(
            serde_json::to_string(&imax).unwrap(),
            r#"{"theatre":{"projection":{"imax":"4k_dual_laser"},"screen":"regular","three_d":true,"hfr":true,"dbox":true}}"#
        );

        let h = Format::Home {
            medium: HomeMedium::Custom("d-box".into()),
            three_d: false,
            hfr: false,
        };
        assert_eq!(
            serde_json::to_string(&h).unwrap(),
            r#"{"home":{"medium":{"custom":"d-box"},"three_d":false,"hfr":false}}"#
        );

        assert_eq!(dci.venue(), Venue::Theatre);
        assert_eq!(h.venue(), Venue::Home);

        // The two IMAX 4K laser variants are distinct (different screens).
        assert_ne!(Imax::Laser4KSingle, Imax::Laser4KDual);
    }

    #[test]
    fn venue_format_mismatch() {
        let res = WatchLog::with_format_now(
            FilmId::Imdb("tt0137523".into()),
            "Fight Club",
            Venue::Home,
            Format::Theatre {
                projection: Projection::Film(Film::Mm70),
                screen: ScreenClass::Regular,
                three_d: false,
                hfr: false,
                dbox: false,
            },
        );
        assert!(matches!(res, Err(Error::VenueFormatMismatch { .. })));

        let mut ok = WatchLog::with_format_now(
            FilmId::Imdb("tt0137523".into()),
            "Fight Club",
            Venue::Theatre,
            Format::Theatre {
                projection: Projection::Film(Film::Mm70),
                screen: ScreenClass::Regular,
                three_d: false,
                hfr: false,
                dbox: false,
            },
        )
        .unwrap();

        // A format swapped to a different venue after construction fails
        // validation too.
        ok.format = Some(Format::Home {
            medium: HomeMedium::BluRay,
            three_d: false,
            hfr: false,
        });
        assert!(matches!(
            ok.validate(),
            Err(Error::VenueFormatMismatch { .. })
        ));
    }

    #[test]
    fn validate_requires_format() {
        let log = WatchLog::now(
            FilmId::Imdb("tt0137523".into()),
            "Fight Club",
            Venue::Theatre,
        );
        assert!(matches!(log.validate(), Err(Error::MissingFormat { .. })));
    }

    #[test]
    fn projection_allows_flags() {
        let dci = Projection::Dci(Dci::Laser4K);
        assert!(dci.allows(Flag::ThreeD));
        assert!(dci.allows(Flag::Hfr));
        assert!(dci.allows(Flag::Dbox));

        let imax_4k = Projection::Imax(Imax::Laser4KDual);
        assert!(imax_4k.allows(Flag::ThreeD));
        assert!(imax_4k.allows(Flag::Hfr));
        assert!(!imax_4k.allows(Flag::Dbox));

        let imax_70mm = Projection::Imax(Imax::Film70mm);
        assert!(!imax_70mm.allows(Flag::ThreeD));
        assert!(!imax_70mm.allows(Flag::Hfr));
        assert!(!imax_70mm.allows(Flag::Dbox));

        let film = Projection::Film(Film::Mm35);
        assert!(film.allows(Flag::ThreeD));
        assert!(!film.allows(Flag::Hfr));
        assert!(!film.allows(Flag::Dbox));
    }

    #[test]
    fn screen_allows_and_forces_flags() {
        let regular = ScreenClass::Regular;
        assert!(regular.allows(Flag::ThreeD));
        assert!(regular.allows(Flag::Hfr));
        assert!(regular.allows(Flag::Dbox));
        assert!(!regular.forces(Flag::ThreeD));

        let vip = ScreenClass::Vip;
        assert!(vip.allows(Flag::ThreeD));
        assert!(vip.allows(Flag::Hfr));
        assert!(!vip.allows(Flag::Dbox));

        let four_dx = ScreenClass::FourDx;
        assert!(four_dx.allows(Flag::ThreeD));
        assert!(four_dx.allows(Flag::Hfr));
        assert!(!four_dx.allows(Flag::Dbox));
        assert!(four_dx.forces(Flag::ThreeD));
        assert!(!four_dx.forces(Flag::Hfr));

        assert!(ScreenClass::Ultra4Dx.forces(Flag::ThreeD));

        let screen_x = ScreenClass::ScreenX;
        assert!(!screen_x.allows(Flag::ThreeD));
        assert!(!screen_x.allows(Flag::Hfr));
        assert!(!screen_x.allows(Flag::Dbox));

        let imax_classic = ScreenClass::ImaxClassic;
        assert!(imax_classic.allows(Flag::ThreeD));
        assert!(imax_classic.allows(Flag::Hfr));
        assert!(!imax_classic.allows(Flag::Dbox));
    }

    #[test]
    fn imax_aspect_ratio() {
        assert_eq!(ScreenClass::ImaxClassic.aspect_ratio(), Some("1.43:1"));
        assert_eq!(ScreenClass::ImaxRetrofit.aspect_ratio(), Some("1.90:1"));
        assert_eq!(ScreenClass::Regular.aspect_ratio(), None);
    }

    #[test]
    fn projection_runs_on_screen() {
        let imax = Projection::Imax(Imax::Laser4KDual);
        assert!(imax.runs_on(&ScreenClass::ImaxClassic));
        assert!(imax.runs_on(&ScreenClass::ImaxRetrofit));
        assert!(!imax.runs_on(&ScreenClass::Regular));
        assert!(!imax.runs_on(&ScreenClass::Plf));

        let dci = Projection::Dci(Dci::Laser4K);
        assert!(dci.runs_on(&ScreenClass::Regular));
        assert!(dci.runs_on(&ScreenClass::Plf));
        assert!(dci.runs_on(&ScreenClass::Vip));
        assert!(dci.runs_on(&ScreenClass::FourDx));
        assert!(dci.runs_on(&ScreenClass::ScreenX));
        assert!(dci.runs_on(&ScreenClass::Ultra4Dx));
        assert!(!dci.runs_on(&ScreenClass::ImaxClassic));
        assert!(!dci.runs_on(&ScreenClass::ImaxRetrofit));

        let film = Projection::Film(Film::Mm70);
        assert!(film.runs_on(&ScreenClass::Regular));
        assert!(!film.runs_on(&ScreenClass::Plf));
        assert!(!film.runs_on(&ScreenClass::ImaxClassic));
    }

    #[test]
    fn validate_theatre_ok() {
        let ok = Format::Theatre {
            projection: Projection::Dci(Dci::Laser4K),
            screen: ScreenClass::Plf,
            three_d: true,
            hfr: true,
            dbox: true,
        };
        assert!(ok.validate().is_ok());

        let home = Format::Home {
            medium: HomeMedium::BluRay,
            three_d: false,
            hfr: false,
        };
        assert!(home.validate().is_ok());
    }

    #[test]
    fn validate_theatre_projection_screen_mismatch() {
        let bad = Format::Theatre {
            projection: Projection::Imax(Imax::Laser4KDual),
            screen: ScreenClass::Plf,
            three_d: false,
            hfr: false,
            dbox: false,
        };
        assert!(matches!(
            bad.validate(),
            Err(Error::ProjectionScreenMismatch { .. })
        ));

        let bad2 = Format::Theatre {
            projection: Projection::Dci(Dci::Laser4K),
            screen: ScreenClass::ImaxRetrofit,
            three_d: false,
            hfr: false,
            dbox: false,
        };
        assert!(matches!(
            bad2.validate(),
            Err(Error::ProjectionScreenMismatch { .. })
        ));
    }

    #[test]
    fn validate_theatre_forced_flag_missing() {
        let bad = Format::Theatre {
            projection: Projection::Dci(Dci::Laser4K),
            screen: ScreenClass::FourDx,
            three_d: false,
            hfr: false,
            dbox: false,
        };
        assert!(matches!(
            bad.validate(),
            Err(Error::FlagMismatch {
                flag: Flag::ThreeD,
                ..
            })
        ));
    }

    #[test]
    fn validate_theatre_flag_not_allowed() {
        let bad = Format::Theatre {
            projection: Projection::Dci(Dci::Laser4K),
            screen: ScreenClass::Vip,
            three_d: false,
            hfr: false,
            dbox: true,
        };
        assert!(matches!(
            bad.validate(),
            Err(Error::FlagMismatch {
                flag: Flag::Dbox,
                ..
            })
        ));
    }

    #[test]
    fn with_format_propagates_validation() {
        let bad = WatchLog::with_format_now(
            FilmId::Imdb("tt0137523".into()),
            "Fight Club",
            Venue::Theatre,
            Format::Theatre {
                projection: Projection::Imax(Imax::Laser4KDual),
                screen: ScreenClass::Plf,
                three_d: false,
                hfr: false,
                dbox: false,
            },
        );
        assert!(matches!(bad, Err(Error::ProjectionScreenMismatch { .. })));

        let ok = WatchLog::with_format_now(
            FilmId::Imdb("tt0137523".into()),
            "Fight Club",
            Venue::Theatre,
            Format::Theatre {
                projection: Projection::Dci(Dci::Laser4K),
                screen: ScreenClass::Plf,
                three_d: true,
                hfr: false,
                dbox: false,
            },
        );
        assert!(ok.is_ok());
    }

    #[test]
    fn rating_serde_roundtrip() {
        assert_eq!(serde_json::to_string(&Rating::Up).unwrap(), r#""up""#);
        assert_eq!(serde_json::to_string(&Rating::Down).unwrap(), r#""down""#);
        let up: Rating = serde_json::from_str(r#""up""#).unwrap();
        assert_eq!(up, Rating::Up);
        let down: Rating = serde_json::from_str(r#""down""#).unwrap();
        assert_eq!(down, Rating::Down);
    }

    #[test]
    fn watch_log_details_roundtrip() {
        let log = WatchLog::now(
            FilmId::Imdb("tt0137523".into()),
            "Fight Club",
            Venue::Theatre,
        )
        .with_rating(Rating::Up);
        assert_eq!(log.rating, Some(Rating::Up));
        assert!(!log.rewatch);
        assert_eq!(log.watch_count, None);

        let json = serde_json::to_string(&log).unwrap();
        let parsed: WatchLog = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, log);
    }

    #[test]
    fn validate_watch_count_range() {
        let mut log = WatchLog::now(
            FilmId::Imdb("tt0137523".into()),
            "Fight Club",
            Venue::Theatre,
        );
        log.format = Some(Format::Theatre {
            projection: Projection::Dci(Dci::Laser4K),
            screen: ScreenClass::Plf,
            three_d: false,
            hfr: false,
            dbox: false,
        });
        assert!(log.validate().is_ok());
        log.watch_count = Some(0);
        assert!(matches!(
            log.validate(),
            Err(Error::InvalidWatchCount { .. })
        ));
        log.watch_count = Some(3);
        assert!(log.validate().is_ok());
    }

    #[test]
    fn rating_boards_cover_tmdb_regions() {
        let boards = super::rating_boards();
        assert_eq!(boards.len(), 52);
        assert_eq!(
            super::rating_board("mpa").map(|board| board.region),
            Some("US".into())
        );
        assert_eq!(
            super::rating_board("ca-qc").map(|board| board.name),
            Some("Canada (Quebec)".into())
        );
    }

    #[test]
    fn certification_lookup_and_roundtrip() {
        let mut certifications = Certifications::default();
        certifications.insert("us", "R");
        certifications.insert("gb", "15");
        certifications.insert("de", "FSK 16");
        certifications.insert("jp", "R12+");
        assert_eq!(certifications.get("US"), Some("R"));
        assert_eq!(certifications.get("GB"), Some("15"));
        assert_eq!(certifications.get("DE"), Some("FSK 16"));
        assert_eq!(certifications.get("JP"), Some("R12+"));

        let json = serde_json::to_string(&certifications).unwrap();
        let parsed: Certifications = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, certifications);

        // Legacy exports stored the three original fields by name.
        let legacy: Certifications =
            serde_json::from_str(r#"{"us":"R","gb":"15","de":null}"#).unwrap();
        assert_eq!(legacy.get("US"), Some("R"));
        assert_eq!(legacy.get("GB"), Some("15"));
        assert_eq!(legacy.get("DE"), None);
    }

    #[test]
    fn settings_default_and_serde_default() {
        assert_eq!(Settings::default().selected_board, "US");
        // A document without the field (an older log) deserializes to the default.
        let parsed: Settings = serde_json::from_str("{}").unwrap();
        assert_eq!(parsed, Settings::default());
        // Legacy board names map to region codes.
        let legacy: Settings = serde_json::from_str(r#"{"selected_board":"fsk"}"#).unwrap();
        assert_eq!(legacy.selected_board, "DE");
        let settings = Settings {
            selected_board: "GB".to_owned(),
        };
        let json = serde_json::to_string(&settings).unwrap();
        let parsed: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, settings);
    }
}
