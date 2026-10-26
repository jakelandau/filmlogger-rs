//! Domain types for the watch log.
//!
//! These are the stable, serializable core of the library: a [`WatchLog`]
//! records one viewing of a [`FilmId`], in a [`Venue`], at a UTC instant,
//! with an optional viewing [`Format`]. Films are identified by IMDB ID when
//! known, falling back to the TMDB numeric ID.

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

impl Serialize for FilmId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            FilmId::Imdb(s) => serializer.serialize_str(s),
            FilmId::Tmdb(n) => serializer.serialize_str(&format!("tmdb:{n}")),
        }
    }
}

impl<'de> Deserialize<'de> for FilmId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if let Some(n) = s.strip_prefix("tmdb:") {
            let parsed = n
                .parse::<i64>()
                .map_err(|_| serde::de::Error::custom(format!("unknown FilmId: {s}")))?;
            Ok(FilmId::Tmdb(parsed))
        } else {
            Ok(FilmId::Imdb(s))
        }
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
    },
    /// A home viewing.
    #[serde(rename = "home")]
    Home {
        /// The release medium.
        medium: HomeMedium,
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
}

/// Theatrical projection technology.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Projection {
    /// Digital projector, lamp bulb, 2K.
    #[serde(rename = "lamp_2k")]
    Lamp2K,
    /// Digital projector, lamp bulb, 4K.
    #[serde(rename = "lamp_4k")]
    Lamp4K,
    /// Digital projector, laser, 2K.
    #[serde(rename = "laser_2k")]
    Laser2K,
    /// Digital projector, laser, 4K.
    #[serde(rename = "laser_4k")]
    Laser4K,
    /// Film projector, 35mm.
    #[serde(rename = "film_35mm")]
    Film35mm,
    /// Film projector, standard 70mm.
    #[serde(rename = "film_70mm")]
    Film70mm,
    /// Film projector, IMAX 70mm.
    #[serde(rename = "film_imax_70mm")]
    FilmImax70mm,
    /// A projection technology not listed above.
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
    /// An auditorium class not listed above.
    #[serde(rename = "custom")]
    Custom(String),
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
    /// 4K Ultra HD Blu-ray.
    #[serde(rename = "uhd_bluray")]
    UhdBluRay,
    /// `LaserDisc`.
    #[serde(rename = "laserdisc")]
    LaserDisc,
    /// Streaming.
    #[serde(rename = "streaming")]
    Streaming,
    /// A medium not listed above.
    #[serde(rename = "custom")]
    Custom(String),
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
    /// Where it was watched.
    pub venue: Venue,
    /// The instant the film was watched, in UTC.
    pub watched_at: DateTime<Utc>,
    /// Free-form notes.
    pub notes: Option<String>,
    /// How it was viewed, if known.
    pub format: Option<Format>,
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
            venue,
            watched_at: local.with_timezone(&Utc),
            notes: None,
            format: None,
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
    /// `title`) are filled from the movie.
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
            venue,
            watched_at: local.with_timezone(&Utc),
            notes: None,
            format: None,
        }
    }

    /// Convenience: [`Self::from_movie`] with [`Local::now`].
    #[must_use]
    pub fn from_movie_now(movie: &Movie, venue: Venue) -> Self {
        Self::from_movie(movie, venue, Local::now())
    }

    /// Create a log with a known viewing format.
    ///
    /// # Errors
    ///
    /// Returns [`Error::VenueFormatMismatch`] if `format.venue() != venue`.
    ///
    /// # Panics
    ///
    /// Panics if [`Format`] cannot be serialized to JSON, which should not
    /// occur for a valid format.
    pub fn with_format(
        film_id: FilmId,
        title: impl Into<String>,
        venue: Venue,
        format: Format,
        local: DateTime<Local>,
    ) -> Result<Self, Error> {
        if format.venue() != venue {
            return Err(Error::VenueFormatMismatch {
                venue,
                format: serde_json::to_string(&format).unwrap(),
            });
        }
        let mut log = Self::new(film_id, title, venue, local);
        log.format = Some(format);
        Ok(log)
    }

    /// Convenience: [`Self::with_format`] with [`Local::now`].
    ///
    /// # Errors
    ///
    /// As [`Self::with_format`].
    ///
    /// # Panics
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
    /// Returns [`Error::VenueFormatMismatch`] if `format.venue() != venue`.
    ///
    /// # Panics
    ///
    /// Panics if [`Format`] cannot be serialized to JSON, which should not
    /// occur for a valid format.
    pub fn from_movie_with_format(
        movie: &Movie,
        venue: Venue,
        format: Format,
        local: DateTime<Local>,
    ) -> Result<Self, Error> {
        if format.venue() != venue {
            return Err(Error::VenueFormatMismatch {
                venue,
                format: serde_json::to_string(&format).unwrap(),
            });
        }
        let mut log = Self::from_movie(movie, venue, local);
        log.format = Some(format);
        Ok(log)
    }

    /// Convenience: [`Self::from_movie_with_format`] with [`Local::now`].
    ///
    /// # Errors
    ///
    /// As [`Self::from_movie_with_format`].
    ///
    /// # Panics
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
}

#[cfg(test)]
mod tests {
    use super::{FilmId, Format, HomeMedium, Projection, ScreenClass, Venue, WatchLog};
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
        let f = Format::Theatre {
            projection: Projection::Laser4K,
            screen: ScreenClass::Plf,
        };
        assert_eq!(
            serde_json::to_string(&f).unwrap(),
            r#"{"theatre":{"projection":"laser_4k","screen":"plf"}}"#
        );
        let h = Format::Home {
            medium: HomeMedium::Custom("d-box".into()),
        };
        assert_eq!(
            serde_json::to_string(&h).unwrap(),
            r#"{"home":{"medium":{"custom":"d-box"}}}"#
        );
        assert_eq!(f.venue(), Venue::Theatre);
        assert_eq!(h.venue(), Venue::Home);
    }

    #[test]
    fn venue_format_mismatch() {
        let res = WatchLog::with_format_now(
            FilmId::Imdb("tt0137523".into()),
            "Fight Club",
            Venue::Home,
            Format::Theatre {
                projection: Projection::Film70mm,
                screen: ScreenClass::Regular,
            },
        );
        assert!(matches!(res, Err(Error::VenueFormatMismatch { .. })));

        let ok = WatchLog::with_format_now(
            FilmId::Imdb("tt0137523".into()),
            "Fight Club",
            Venue::Theatre,
            Format::Theatre {
                projection: Projection::Film70mm,
                screen: ScreenClass::Regular,
            },
        );
        assert!(ok.is_ok());
    }
}
