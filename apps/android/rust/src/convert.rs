//! Conversions between [`film_logger`] core types and this crate's uniffi DTOs.

use chrono::{DateTime, Utc};

use crate::types::{
    CastMember, Certification, Certifications, CollectionRef, CountEntry, Dci, Film, FilmId,
    FilmListEntry, FilmMetadata, FilmPage, FilmStats, FilmViewings, Format, FormatCatalog,
    FormatGroup, FormatLegality, FormatOption, Genre, HomeMedium, Imax, LoggerError, Movie,
    MovieSummary, NewLogDetails, Projection, Rating, RatingBoard, ScreenClass, SearchPage,
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

impl From<core::RatingBoard> for RatingBoard {
    fn from(b: core::RatingBoard) -> Self {
        Self {
            region: b.region,
            name: b.name,
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
            Format::Home { medium } => core::Format::Home {
                medium: medium.into(),
            },
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
            core::Format::Home { medium } => Format::Home {
                medium: medium.into(),
            },
            _ => unreachable!(),
        }
    }
}

// --- Certifications -------------------------------------------------------

impl From<Certifications> for core::Certifications {
    fn from(c: Certifications) -> Self {
        let mut certifications = core::Certifications::default();
        for value in c.values {
            certifications.insert(value.region, value.rating);
        }
        certifications
    }
}

impl From<core::Certifications> for Certifications {
    fn from(c: core::Certifications) -> Self {
        Self {
            values: c
                .iter()
                .map(|(region, rating)| Certification {
                    region: region.to_owned(),
                    rating: rating.to_owned(),
                })
                .collect(),
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
            timezone: self.timezone.clone(),
            notes: self.notes.clone(),
            rating: self.rating.map(Into::into),
            rewatch: self.rewatch,
            watch_count: self.watch_count,
            format: self.format.clone().map(Into::into),
            format_summary: self.format_summary.clone(),
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
            timezone: l.timezone,
            notes: l.notes,
            rating: l.rating.map(Into::into),
            rewatch: l.rewatch,
            watch_count: l.watch_count,
            format: l.format.map(Into::into),
            format_summary: l.format_summary,
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
            cast: m.cast.into_iter().map(Into::into).collect(),
            collection: m.collection.map(Into::into),
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
            viewings_by_month: count_entries(s.viewings_by_month),
            viewings_by_projection_family: count_entries(s.viewings_by_projection_family),
            viewings_by_screen: count_entries(s.viewings_by_screen),
            viewings_by_medium: count_entries(s.viewings_by_medium),
            viewings_by_flag: count_entries(s.viewings_by_flag),
        }
    }
}

/// Convert a breakdown map into ordered `(key, count)` entries.
fn count_entries(map: std::collections::BTreeMap<String, usize>) -> Vec<CountEntry> {
    map.into_iter()
        .map(|(key, count)| CountEntry {
            key,
            #[allow(clippy::cast_possible_truncation)]
            count: count as u32,
        })
        .collect()
}

// --- Settings ------------------------------------------------------------

impl From<core::Settings> for Settings {
    fn from(s: core::Settings) -> Self {
        Self {
            selected_board: s.selected_board,
        }
    }
}

// --- Films and the library ------------------------------------------------

impl From<core::CastMember> for CastMember {
    fn from(c: core::CastMember) -> Self {
        Self {
            id: c.id,
            name: c.name,
            character: c.character,
            order: c.order,
        }
    }
}

impl From<core::CollectionRef> for CollectionRef {
    fn from(c: core::CollectionRef) -> Self {
        Self {
            id: c.id,
            name: c.name,
            poster_path: c.poster_path,
        }
    }
}

impl From<core::FilmMetadata> for FilmMetadata {
    fn from(f: core::FilmMetadata) -> Self {
        Self {
            film_id: f.film_id.into(),
            tmdb_id: f.tmdb_id,
            imdb_id: f.imdb_id,
            title: f.title,
            original_title: f.original_title,
            release_date: f.release_date,
            release_year: f.release_year,
            runtime_minutes: f.runtime_minutes,
            overview: f.overview,
            tagline: f.tagline,
            original_language: f.original_language,
            poster_path: f.poster_path,
            backdrop_path: f.backdrop_path,
            vote_average: f.vote_average,
            vote_count: f.vote_count,
            popularity: f.popularity,
            adult: f.adult,
            genres: f.genres.into_iter().map(Into::into).collect(),
            cast: f.cast.into_iter().map(Into::into).collect(),
            certifications: f.certifications.map(Into::into),
            collection: f.collection.map(Into::into),
            favorite: f.favorite,
            watchlisted: f.watchlisted,
            metadata_refreshed_at: f.metadata_refreshed_at.map(|d| d.to_rfc3339()),
        }
    }
}

impl From<core::FilmListEntry> for FilmListEntry {
    fn from(e: core::FilmListEntry) -> Self {
        Self {
            film: e.film.into(),
            #[allow(clippy::cast_possible_truncation)]
            viewings: e.viewings as u32,
            last_watched_at: e.last_watched_at.map(|d| d.to_rfc3339()),
        }
    }
}

impl From<core::FilmStats> for FilmStats {
    fn from(s: core::FilmStats) -> Self {
        Self {
            #[allow(clippy::cast_possible_truncation)]
            viewings: s.viewings as u32,
            #[allow(clippy::cast_possible_truncation)]
            theatre: s.theatre as u32,
            #[allow(clippy::cast_possible_truncation)]
            home: s.home as u32,
            first_watched_at: s.first_watched_at.map(|d| d.to_rfc3339()),
            last_watched_at: s.last_watched_at.map(|d| d.to_rfc3339()),
            formats: count_entries(s.formats),
        }
    }
}

impl From<core::FilmPage> for FilmPage {
    fn from(p: core::FilmPage) -> Self {
        Self {
            film: p.film.into(),
            logs: p.logs.into_iter().map(Into::into).collect(),
            stats: p.stats.into(),
        }
    }
}

// --- Format catalog --------------------------------------------------------

impl From<core::FormatOption> for FormatOption {
    fn from(o: core::FormatOption) -> Self {
        Self {
            key: o.key,
            label: o.label,
        }
    }
}

impl From<core::FormatLegality> for FormatLegality {
    fn from(l: core::FormatLegality) -> Self {
        Self {
            family: l.family,
            technology: l.technology,
            screen: l.screen,
            allowed_flags: l.allowed_flags,
            forced_flags: l.forced_flags,
        }
    }
}

/// Convert a catalog's grouped options into ordered groups.
fn format_groups(
    groups: std::collections::BTreeMap<String, Vec<core::FormatOption>>,
) -> Vec<FormatGroup> {
    groups
        .into_iter()
        .map(|(key, options)| FormatGroup {
            key,
            options: options.into_iter().map(Into::into).collect(),
        })
        .collect()
}

impl From<core::FormatCatalog> for FormatCatalog {
    fn from(c: core::FormatCatalog) -> Self {
        Self {
            families: c.families.into_iter().map(Into::into).collect(),
            technologies: format_groups(c.technologies),
            screens: format_groups(c.screens),
            mediums: c.mediums.into_iter().map(Into::into).collect(),
            flag_labels: c.flag_labels.into_iter().map(Into::into).collect(),
            legal: c.legal.into_iter().map(Into::into).collect(),
        }
    }
}

impl NewLogDetails {
    /// Convert this DTO into the core details, parsing `watched_at`.
    ///
    /// # Errors
    ///
    /// Returns [`LoggerError::Serde`] if `watched_at` is not a valid RFC 3339
    /// timestamp.
    pub fn to_core(&self) -> Result<core::NewLogDetails, LoggerError> {
        let watched_at = match &self.watched_at {
            Some(value) => Some(
                DateTime::parse_from_rfc3339(value)
                    .map(|d| d.with_timezone(&Utc))
                    .map_err(|e| LoggerError::Serde {
                        detail: format!("invalid watched_at: {e}"),
                    })?,
            ),
            None => None,
        };
        Ok(core::NewLogDetails {
            watched_at,
            timezone: self.timezone.clone(),
            notes: self.notes.clone(),
            rating: self.rating.map(Into::into),
        })
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
            core::Error::VenueFormatMismatch { venue, format } => {
                LoggerError::VenueFormatMismatch {
                    venue: venue.into(),
                    format: serde_json::to_string(&format).unwrap_or_default(),
                }
            }
            core::Error::ProjectionScreenMismatch { projection, screen } => {
                LoggerError::ProjectionScreenMismatch {
                    detail: format!("projection {projection:?} does not run in screen {screen:?}"),
                }
            }
            core::Error::FlagMismatch {
                flag,
                projection,
                screen,
            } => LoggerError::FlagMismatch {
                detail: format!(
                    "flag {flag:?} does not match projection {projection:?} in screen {screen:?}"
                ),
            },
            core::Error::InvalidWatchCount { count } => LoggerError::InvalidWatchCount { count },
            core::Error::InvalidImport { detail } => LoggerError::InvalidImport { detail },
            core::Error::NoTmdbId { film_id } => LoggerError::NoTmdbId {
                film_id: film_id.into(),
            },
            core::Error::InvalidFilmId { value } => LoggerError::InvalidFilmId { value },
            core::Error::InvalidStoredLog { id } => LoggerError::InvalidStoredLog { id },
        }
    }
}
