//! Conversions between [`film_logger`] core types and this crate's uniffi DTOs.

use chrono::{DateTime, Utc};

use crate::types::{
    Certifications, Dci, Film, FilmId, FilmViewings, Format, Genre, HomeMedium, Imax,
    LoggerError, Movie, MovieSummary, Projection, Rating, RatingBoard, ScreenClass, SearchPage,
    Settings, Stats, Venue, VenueCounts, WatchLog, YearCount,
};
use film_logger as core;

// --- Venue -----------------------------------------------------------------

impl From<Venue> for core::Venue {
    fn from(v: Venue) -> Self {
        match v {
            Venue::Theatre => core::Venue::Theatre,
            Venue::Home => core::Venue::Home,
        }
    }
}

impl From<core::Venue> for Venue {
    fn from(v: core::Venue) -> Self {
        match v {
            core::Venue::Theatre => Venue::Theatre,
            core::Venue::Home => Venue::Home,
        }
    }
}

// --- Rating ----------------------------------------------------------------

impl From<Rating> for core::Rating {
    fn from(r: Rating) -> Self {
        match r {
            Rating::Up => core::Rating::Up,
            Rating::Down => core::Rating::Down,
        }
    }
}

impl From<core::Rating> for Rating {
    fn from(r: core::Rating) -> Self {
        match r {
            core::Rating::Up => Rating::Up,
            core::Rating::Down => Rating::Down,
            _ => unreachable!(),
        }
    }
}

// --- RatingBoard ----------------------------------------------------------

impl From<RatingBoard> for core::RatingBoard {
    fn from(b: RatingBoard) -> Self {
        match b {
            RatingBoard::Mpa => core::RatingBoard::Mpa,
            RatingBoard::Bbfc => core::RatingBoard::Bbfc,
            RatingBoard::Fsk => core::RatingBoard::Fsk,
        }
    }
}

impl From<core::RatingBoard> for RatingBoard {
    fn from(b: core::RatingBoard) -> Self {
        match b {
            core::RatingBoard::Mpa => RatingBoard::Mpa,
            core::RatingBoard::Bbfc => RatingBoard::Bbfc,
            core::RatingBoard::Fsk => RatingBoard::Fsk,
            _ => unreachable!(),
        }
    }
}

// --- FilmId ---------------------------------------------------------------

impl From<FilmId> for core::FilmId {
    fn from(f: FilmId) -> Self {
        match f {
            FilmId::Imdb { id } => core::FilmId::Imdb(id),
            FilmId::Tmdb { id } => core::FilmId::Tmdb(id),
        }
    }
}

impl From<core::FilmId> for FilmId {
    fn from(f: core::FilmId) -> Self {
        match f {
            core::FilmId::Imdb(id) => FilmId::Imdb { id },
            core::FilmId::Tmdb(id) => FilmId::Tmdb { id },
        }
    }
}

// --- Dci -------------------------------------------------------------------

impl From<Dci> for core::Dci {
    fn from(d: Dci) -> Self {
        match d {
            Dci::Lamp2K => core::Dci::Lamp2K,
            Dci::Laser2K => core::Dci::Laser2K,
            Dci::Lamp4K => core::Dci::Lamp4K,
            Dci::Laser4K => core::Dci::Laser4K,
            Dci::Custom { id } => core::Dci::Custom(id),
        }
    }
}

impl From<core::Dci> for Dci {
    fn from(d: core::Dci) -> Self {
        match d {
            core::Dci::Lamp2K => Dci::Lamp2K,
            core::Dci::Laser2K => Dci::Laser2K,
            core::Dci::Lamp4K => Dci::Lamp4K,
            core::Dci::Laser4K => Dci::Laser4K,
            core::Dci::Custom(id) => Dci::Custom { id },
            _ => unreachable!(),
        }
    }
}

// --- Imax ------------------------------------------------------------------

impl From<Imax> for core::Imax {
    fn from(i: Imax) -> Self {
        match i {
            Imax::Xenon2K => core::Imax::Xenon2K,
            Imax::Laser4KSingle => core::Imax::Laser4KSingle,
            Imax::Laser4KDual => core::Imax::Laser4KDual,
            Imax::Film70mm => core::Imax::Film70mm,
            Imax::Custom { id } => core::Imax::Custom(id),
        }
    }
}

impl From<core::Imax> for Imax {
    fn from(i: core::Imax) -> Self {
        match i {
            core::Imax::Xenon2K => Imax::Xenon2K,
            core::Imax::Laser4KSingle => Imax::Laser4KSingle,
            core::Imax::Laser4KDual => Imax::Laser4KDual,
            core::Imax::Film70mm => Imax::Film70mm,
            core::Imax::Custom(id) => Imax::Custom { id },
            _ => unreachable!(),
        }
    }
}

// --- Film ------------------------------------------------------------------

impl From<Film> for core::Film {
    fn from(f: Film) -> Self {
        match f {
            Film::Mm35 => core::Film::Mm35,
            Film::Mm70 => core::Film::Mm70,
            Film::Custom { id } => core::Film::Custom(id),
        }
    }
}

impl From<core::Film> for Film {
    fn from(f: core::Film) -> Self {
        match f {
            core::Film::Mm35 => Film::Mm35,
            core::Film::Mm70 => Film::Mm70,
            core::Film::Custom(id) => Film::Custom { id },
            _ => unreachable!(),
        }
    }
}

// --- ScreenClass ----------------------------------------------------------

impl From<ScreenClass> for core::ScreenClass {
    fn from(s: ScreenClass) -> Self {
        match s {
            ScreenClass::Regular => core::ScreenClass::Regular,
            ScreenClass::Plf => core::ScreenClass::Plf,
            ScreenClass::Vip => core::ScreenClass::Vip,
            ScreenClass::FourDx => core::ScreenClass::FourDx,
            ScreenClass::ScreenX => core::ScreenClass::ScreenX,
            ScreenClass::ImaxClassic => core::ScreenClass::ImaxClassic,
            ScreenClass::ImaxRetrofit => core::ScreenClass::ImaxRetrofit,
            ScreenClass::Ultra4Dx => core::ScreenClass::Ultra4Dx,
            ScreenClass::Custom { id } => core::ScreenClass::Custom(id),
        }
    }
}

impl From<core::ScreenClass> for ScreenClass {
    fn from(s: core::ScreenClass) -> Self {
        match s {
            core::ScreenClass::Regular => ScreenClass::Regular,
            core::ScreenClass::Plf => ScreenClass::Plf,
            core::ScreenClass::Vip => ScreenClass::Vip,
            core::ScreenClass::FourDx => ScreenClass::FourDx,
            core::ScreenClass::ScreenX => ScreenClass::ScreenX,
            core::ScreenClass::ImaxClassic => ScreenClass::ImaxClassic,
            core::ScreenClass::ImaxRetrofit => ScreenClass::ImaxRetrofit,
            core::ScreenClass::Ultra4Dx => ScreenClass::Ultra4Dx,
            core::ScreenClass::Custom(id) => ScreenClass::Custom { id },
            _ => unreachable!(),
        }
    }
}

// --- HomeMedium ------------------------------------------------------------

impl From<HomeMedium> for core::HomeMedium {
    fn from(m: HomeMedium) -> Self {
        match m {
            HomeMedium::Dvd => core::HomeMedium::Dvd,
            HomeMedium::HdDvd => core::HomeMedium::HdDvd,
            HomeMedium::BluRay => core::HomeMedium::BluRay,
            HomeMedium::UhdBluRay => core::HomeMedium::UhdBluRay,
            HomeMedium::LaserDisc => core::HomeMedium::LaserDisc,
            HomeMedium::Streaming => core::HomeMedium::Streaming,
            HomeMedium::Broadcast => core::HomeMedium::Broadcast,
            HomeMedium::Telecine => core::HomeMedium::Telecine,
            HomeMedium::Custom { id } => core::HomeMedium::Custom(id),
        }
    }
}

impl From<core::HomeMedium> for HomeMedium {
    fn from(m: core::HomeMedium) -> Self {
        match m {
            core::HomeMedium::Dvd => HomeMedium::Dvd,
            core::HomeMedium::HdDvd => HomeMedium::HdDvd,
            core::HomeMedium::BluRay => HomeMedium::BluRay,
            core::HomeMedium::UhdBluRay => HomeMedium::UhdBluRay,
            core::HomeMedium::LaserDisc => HomeMedium::LaserDisc,
            core::HomeMedium::Streaming => HomeMedium::Streaming,
            core::HomeMedium::Broadcast => HomeMedium::Broadcast,
            core::HomeMedium::Telecine => HomeMedium::Telecine,
            core::HomeMedium::Custom(id) => HomeMedium::Custom { id },
            _ => unreachable!(),
        }
    }
}

// --- Projection ------------------------------------------------------------

impl From<Projection> for core::Projection {
    fn from(p: Projection) -> Self {
        match p {
            Projection::Dci { dci } => core::Projection::Dci(dci.into()),
            Projection::Imax { imax } => core::Projection::Imax(imax.into()),
            Projection::Film { film } => core::Projection::Film(film.into()),
        }
    }
}

impl From<core::Projection> for Projection {
    fn from(p: core::Projection) -> Self {
        match p {
            core::Projection::Dci(dci) => Projection::Dci { dci: dci.into() },
            core::Projection::Imax(imax) => Projection::Imax { imax: imax.into() },
            core::Projection::Film(film) => Projection::Film { film: film.into() },
            _ => unreachable!(),
        }
    }
}

// --- Format ----------------------------------------------------------------

impl From<Format> for core::Format {
    fn from(f: Format) -> Self {
        match f {
            Format::Theatre {
                projection,
                screen,
                three_d,
                hfr,
                dbox,
            } => core::Format::Theatre {
                projection: projection.into(),
                screen: screen.into(),
                three_d,
                hfr,
                dbox,
            },
            Format::Home { medium } => core::Format::Home { medium: medium.into() },
        }
    }
}

impl From<core::Format> for Format {
    fn from(f: core::Format) -> Self {
        match f {
            core::Format::Theatre {
                projection,
                screen,
                three_d,
                hfr,
                dbox,
            } => Format::Theatre {
                projection: projection.into(),
                screen: screen.into(),
                three_d,
                hfr,
                dbox,
            },
            core::Format::Home { medium } => Format::Home { medium: medium.into() },
            _ => unreachable!(),
        }
    }
}

// --- Certifications -------------------------------------------------------

impl From<Certifications> for core::Certifications {
    fn from(c: Certifications) -> Self {
        core::Certifications {
            us: c.us,
            gb: c.gb,
            de: c.de,
        }
    }
}

impl From<core::Certifications> for Certifications {
    fn from(c: core::Certifications) -> Self {
        Self {
            us: c.us,
            gb: c.gb,
            de: c.de,
        }
    }
}

// --- WatchLog -------------------------------------------------------------

impl WatchLog {
    /// Convert this DTO back to a core [`core::WatchLog`].
    ///
    /// # Errors
    ///
    /// Returns [`LoggerError::Serde`] if `watched_at` is not a valid RFC 3339
    /// timestamp.
    pub fn to_core(&self) -> Result<core::WatchLog, LoggerError> {
        let watched_at = DateTime::parse_from_rfc3339(&self.watched_at)
            .map(|d| d.with_timezone(&Utc))
            .map_err(|e| LoggerError::Serde {
                detail: format!("invalid watched_at: {e}"),
            })?;
        Ok(core::WatchLog {
            id: self.id,
            film_id: self.film_id.clone().into(),
            imdb_id: self.imdb_id.clone(),
            tmdb_id: self.tmdb_id,
            title: self.title.clone(),
            release_year: self.release_year,
            poster_path: self.poster_path.clone(),
            certifications: self.certifications.clone().map(Into::into),
            venue: self.venue.into(),
            watched_at,
            notes: self.notes.clone(),
            rating: self.rating.map(Into::into),
            rewatch: self.rewatch,
            watch_count: self.watch_count,
            format: self.format.clone().map(Into::into),
        })
    }
}

impl From<core::WatchLog> for WatchLog {
    fn from(l: core::WatchLog) -> Self {
        Self {
            id: l.id,
            film_id: l.film_id.into(),
            imdb_id: l.imdb_id,
            tmdb_id: l.tmdb_id,
            title: l.title,
            release_year: l.release_year,
            poster_path: l.poster_path,
            certifications: l.certifications.map(Into::into),
            venue: l.venue.into(),
            watched_at: l.watched_at.to_rfc3339(),
            notes: l.notes,
            rating: l.rating.map(Into::into),
            rewatch: l.rewatch,
            watch_count: l.watch_count,
            format: l.format.map(Into::into),
        }
    }
}

// --- Genre -----------------------------------------------------------------

impl From<core::Genre> for Genre {
    fn from(g: core::Genre) -> Self {
        Self {
            id: g.id,
            name: g.name,
        }
    }
}

// --- Movie ---------------------------------------------------------------

impl From<core::Movie> for Movie {
    fn from(m: core::Movie) -> Self {
        Self {
            id: m.id,
            title: m.title,
            original_title: m.original_title,
            original_language: m.original_language,
            release_date: m.release_date,
            runtime: m.runtime,
            genres: m.genres.into_iter().map(Into::into).collect(),
            overview: m.overview,
            tagline: m.tagline,
            status: m.status,
            poster_path: m.poster_path,
            backdrop_path: m.backdrop_path,
            vote_average: m.vote_average,
            vote_count: m.vote_count,
            imdb_id: m.imdb_id,
            popularity: m.popularity,
            adult: m.adult,
            certifications: m.certifications.map(Into::into),
        }
    }
}

// --- MovieSummary --------------------------------------------------------

impl From<core::MovieSummary> for MovieSummary {
    fn from(m: core::MovieSummary) -> Self {
        Self {
            id: m.id,
            title: m.title,
            original_title: m.original_title,
            original_language: m.original_language,
            release_date: m.release_date,
            overview: m.overview,
            genre_ids: m.genre_ids,
            poster_path: m.poster_path,
            backdrop_path: m.backdrop_path,
            vote_average: m.vote_average,
            vote_count: m.vote_count,
            popularity: m.popularity,
            adult: m.adult,
        }
    }
}

// --- SearchPage ----------------------------------------------------------

impl From<core::SearchPage> for SearchPage {
    fn from(p: core::SearchPage) -> Self {
        Self {
            page: p.page,
            results: p.results.into_iter().map(Into::into).collect(),
            total_pages: p.total_pages,
            total_results: p.total_results,
        }
    }
}

// --- Stats -----------------------------------------------------------------

impl From<core::VenueCounts> for VenueCounts {
    fn from(v: core::VenueCounts) -> Self {
        Self {
            #[allow(clippy::cast_possible_truncation)]
            theatre: v.theatre as u32,
            #[allow(clippy::cast_possible_truncation)]
            home: v.home as u32,
        }
    }
}

impl From<core::FilmViewings> for FilmViewings {
    fn from(f: core::FilmViewings) -> Self {
        Self {
            title: f.title,
            #[allow(clippy::cast_possible_truncation)]
            count: f.count as u32,
        }
    }
}

impl From<core::Stats> for Stats {
    fn from(s: core::Stats) -> Self {
        Self {
            #[allow(clippy::cast_possible_truncation)]
            total_viewings: s.total_viewings as u32,
            #[allow(clippy::cast_possible_truncation)]
            distinct_films: s.distinct_films as u32,
            viewings_by_venue: s.viewings_by_venue.into(),
            viewings_by_year: s
                .viewings_by_year
                .into_iter()
                .map(|(year, count)| YearCount {
                    year,
                    #[allow(clippy::cast_possible_truncation)]
                    count: count as u32,
                })
                .collect(),
            #[allow(clippy::cast_possible_truncation)]
            rewatches: s.rewatches as u32,
            top_films: s.top_films.into_iter().map(Into::into).collect(),
        }
    }
}

// --- Settings ------------------------------------------------------------

impl From<core::Settings> for Settings {
    fn from(s: core::Settings) -> Self {
        Self {
            selected_board: s.selected_board.into(),
        }
    }
}

// --- LoggerError ---------------------------------------------------------

impl From<core::Error> for LoggerError {
    fn from(e: core::Error) -> Self {
        match e {
            core::Error::Io(err) => LoggerError::Io {
                detail: err.to_string(),
            },
            core::Error::Sqlite(err) => LoggerError::Database {
                detail: err.to_string(),
            },
            core::Error::Serde(err) => LoggerError::Serde {
                detail: err.to_string(),
            },
            core::Error::Http(err) => LoggerError::Http {
                detail: err.to_string(),
            },
            core::Error::Api { status, message } => LoggerError::Api {
                status: i32::from(status),
                detail: message,
            },
            core::Error::FilmNotFound => LoggerError::FilmNotFound,
            core::Error::LogNotFound { id } => LoggerError::LogNotFound { id },
            core::Error::SchemaTooNew { version } => LoggerError::SchemaTooNew { version },
            core::Error::NoCompiledApiKey => LoggerError::NoCompiledApiKey,
            core::Error::MissingFormat { venue } => LoggerError::MissingFormat {
                venue: venue.into(),
            },
            core::Error::VenueFormatMismatch { venue, format } => LoggerError::VenueFormatMismatch {
                venue: venue.into(),
                format,
            },
            core::Error::ProjectionScreenMismatch { projection, screen } => {
                LoggerError::ProjectionScreenMismatch {
                    detail: format!(
                        "projection {projection:?} does not run in screen {screen:?}"
                    ),
                }
            }
            core::Error::FlagMismatch {
                flag, projection, screen,
            } => LoggerError::FlagMismatch {
                detail: format!(
                    "flag {flag:?} does not match projection {projection:?} in screen {screen:?}"
                ),
            },
            core::Error::InvalidWatchCount { count } => LoggerError::InvalidWatchCount { count },
        }
    }
}
