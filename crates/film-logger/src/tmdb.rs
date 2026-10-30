//! TMDB API client and models.
//!
//! The client wraps [`tmdb_client`], an OpenAPI-generated client for the
//! TMDB v3 REST API, and re-exposes this crate's own models ([`Movie`],
//! [`MovieSummary`], [`SearchPage`]) as the public API. It fetches a full
//! movie by TMDB id, finds a movie by IMDB id, and searches movies. The
//! TMDB API Key is sent as the `?api_key=` query parameter on every request.
//! Images are referenced by path; the full image URLs are built by
//! [`Movie::poster_url`] and [`Movie::backdrop_url`].

use std::rc::Rc;

use serde::{Deserialize, Serialize};
use tmdb_client::apis::configuration::Configuration;
use tmdb_client::apis::{
    FindApi, FindApiClient, MoviesApi, MoviesApiClient, SearchApi, SearchApiClient,
};
use tmdb_client::models::{MovieDetails, MovieObject, MoviePaginated};

use crate::domain::Certifications;
use crate::error::Error;

/// Base URL of the live TMDB API.
const TMDB_BASE_URL: &str = "https://api.themoviedb.org/3";

/// Base URL of the TMDB image CDN.
const IMAGE_BASE_URL: &str = "https://image.tmdb.org/t/p";

/// The TMDB API key baked in at compile time, if present.
///
/// Set `TMDB_API_KEY` in the build environment (locally: your shell; on
/// GitHub Actions: a repository secret). `build.rs` captures it as a
/// compile-time environment variable, which is embedded here via
/// `option_env!`, so a running process never reads the environment. `None`
/// when the key was absent at build time.
pub static COMPILED_API_KEY: Option<&'static str> = option_env!("TMDB_API_KEY");

/// A genre associated with a film.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Genre {
    /// The TMDB genre id.
    pub id: i32,
    /// The genre name.
    pub name: String,
}

/// A full movie record (TMDB movie detail / find result).
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
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
    /// The board certifications, if fetched from the film's release dates.
    pub certifications: Option<Certifications>,
}

impl Movie {
    /// The release year, parsed from [`Movie::release_date`], if known.
    #[must_use]
    pub fn release_year(&self) -> Option<i32> {
        self.release_date
            .as_deref()
            .and_then(|d| d.get(..4))
            .and_then(|s| s.parse::<i32>().ok())
    }

    /// The full URL for the poster image, if a poster path is known.
    #[must_use]
    pub fn poster_url(&self) -> Option<String> {
        self.poster_path
            .as_deref()
            .map(|p| image_url(p, "original"))
    }

    /// The full URL for the backdrop image, if a backdrop path is known.
    #[must_use]
    pub fn backdrop_url(&self) -> Option<String> {
        self.backdrop_path
            .as_deref()
            .map(|p| image_url(p, "original"))
    }
}

/// A full URL for a TMDB image `path` at `size` (e.g. `"w92"`, `"w342"`,
/// `"w780"`, `"original"`).
#[must_use]
pub fn image_url(path: &str, size: &str) -> String {
    format!("{IMAGE_BASE_URL}/{size}{path}")
}

/// A lightweight movie record (search results).
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
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
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
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

/// The TMDB API client.
///
/// Wraps the generated [`tmdb_client`], re-exposing this crate's models and
/// error type. The underlying client is blocking (the generated client is
/// synchronous), so the methods here are synchronous.
pub struct TmdbClient {
    find: FindApiClient,
    movies: MoviesApiClient,
    search: SearchApiClient,
    /// The API key, for requests this crate makes directly (release dates).
    api_key: String,
    /// The base URL, for requests this crate makes directly.
    base_url: String,
    /// A direct blocking HTTP client, sharing the generated client's TLS.
    raw: reqwest::blocking::Client,
}

/// The generated [`tmdb_client`] `*ApiClient`s hold their configuration in an
/// [`Rc`], which is not [`Send`]. The configuration is written once in
/// [`TmdbClient::make`] and only read afterwards, and the shared
/// `reqwest::blocking::Client` is itself `Send` + `Sync`, so sharing a
/// [`TmdbClient`] across threads is safe.
#[allow(unsafe_code)]
unsafe impl Send for TmdbClient {}
#[allow(unsafe_code)]
unsafe impl Sync for TmdbClient {}

impl TmdbClient {
    /// Create a client for the live TMDB API.
    ///
    /// `api_key` is the TMDB v3 **API Key**, sent as the `?api_key=` query
    /// parameter on every request.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self::make(api_key, TMDB_BASE_URL)
    }

    /// Create a client against a custom base URL (for tests).
    pub fn with_base_url(api_key: impl Into<String>, base_url: impl Into<String>) -> Self {
        Self::make(api_key, base_url)
    }

    /// Create a client from the key baked in at compile time.
    ///
    /// Returns `None` if `TMDB_API_KEY` was absent from the build
    /// environment (see [`COMPILED_API_KEY`]).
    #[must_use]
    pub fn from_compiled_key() -> Option<Self> {
        let api_key = COMPILED_API_KEY?;
        Some(Self::new(api_key))
    }

    fn make(api_key: impl Into<String>, base_url: impl Into<String>) -> Self {
        let api_key = api_key.into();
        let base_url = base_url.into();
        let config = Rc::new({
            let mut config = Configuration::new_with_api_key(api_key.clone());
            config.base_path.clone_from(&base_url);
            config
        });
        Self {
            find: FindApiClient::new(Rc::clone(&config)),
            movies: MoviesApiClient::new(Rc::clone(&config)),
            search: SearchApiClient::new(Rc::clone(&config)),
            api_key,
            base_url,
            raw: config.client.clone(),
        }
    }

    /// Get a full movie by its TMDB numeric id.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Api`] on an HTTP error status (e.g. 404 not found),
    /// [`Error::Serde`] if the response body is not valid JSON, or
    /// [`Error::Http`] on connection, timeout, or other transport errors.
    pub fn movie(&self, id: i64) -> Result<Movie, Error> {
        let details = self
            .movies
            .get_movie_details(Self::tmdb_id(id), Some("en"), None, None)
            .map_err(Self::from_tmdb_error)?;
        let mut movie: Movie = details.into();
        movie.certifications = self.release_certifications(id);
        Ok(movie)
    }

    /// Find a movie by its IMDB id (e.g. `tt0137523`).
    ///
    /// # Errors
    ///
    /// Returns [`Error::FilmNotFound`] if TMDB has no movie for the IMDB id,
    /// [`Error::Api`] on an HTTP error status, [`Error::Serde`] if the
    /// response body is not valid JSON, or [`Error::Http`] on connection,
    /// timeout, or other transport errors.
    pub fn find_by_imdb_id(&self, imdb_id: &str) -> Result<Movie, Error> {
        let find = self
            .find
            .get_find_external_id(imdb_id, "imdb_id", Some("en"))
            .map_err(Self::from_tmdb_error)?;
        let first = find
            .movie_results
            .and_then(|results| results.into_iter().next())
            .ok_or(Error::FilmNotFound)?;
        let id = first.id.ok_or(Error::FilmNotFound)?;
        self.movie(i64::from(id))
    }

    /// Search movies by query.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Api`] on an HTTP error status, [`Error::Serde`] if
    /// the response body is not valid JSON, or [`Error::Http`] on connection,
    /// timeout, or other transport errors.
    pub fn search_movies(&self, query: &str, page: i32) -> Result<SearchPage, Error> {
        let page = self
            .search
            .get_search_movie_paginated(query, None, None, Some("en"), Some(page), None, None)
            .map_err(Self::from_tmdb_error)?;
        Ok(page.into())
    }

    /// The currently popular movies.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Api`] on an HTTP error status, [`Error::Serde`] if
    /// the response body is not valid JSON, or [`Error::Http`] on
    /// connection, timeout, or other transport errors.
    pub fn popular_movies(&self, page: i32) -> Result<SearchPage, Error> {
        let page = self
            .movies
            .get_movie_popular_paginated(Some("en"), Some(page), None)
            .map_err(Self::from_tmdb_error)?;
        Ok(page.into())
    }

    /// Fetch the film's board certifications from its release dates.
    ///
    /// TMDB's release-dates response carries per-region `certification`
    /// fields, which the generated models do not model; this direct request
    /// parses only those. A `None` (rather than `Err`) keeps [`Self::movie`]
    /// resilient when the endpoint is missing for a film.
    fn release_certifications(&self, id: i64) -> Option<Certifications> {
        let url = format!(
            "{}/movie/{id}/release_dates",
            self.base_url,
            id = Self::tmdb_id(id)
        );
        let response: ReleaseDatesResponse = self
            .raw
            .get(url)
            .query(&[("language", "en"), ("api_key", self.api_key.as_str())])
            .send()
            .ok()?
            .error_for_status()
            .ok()?
            .json()
            .ok()?;
        Some(response.certifications())
    }

    /// A TMDB numeric id as an `i32` (ids fit far below `i32::MAX`).
    #[allow(clippy::cast_possible_truncation)]
    fn tmdb_id(id: i64) -> i32 {
        id as i32
    }

    /// Map a [`tmdb_client::Error`] onto this crate's [`Error`].
    fn from_tmdb_error(err: tmdb_client::Error) -> Error {
        match err {
            tmdb_client::Error::Reqwest(re) => match re.status() {
                Some(status) => Error::Api {
                    status: status.as_u16(),
                    message: re.to_string(),
                },
                None => Error::Http(re),
            },
            tmdb_client::Error::Serde(s) => Error::Serde(s),
            tmdb_client::Error::Io(i) => Error::Io(i),
        }
    }
}

/// TMDB's release-dates response: per-region entries whose releases carry
/// board certifications. Parsed directly because the generated models do
/// not carry `certification`.
#[derive(Debug, Deserialize)]
struct ReleaseDatesResponse {
    #[serde(default)]
    results: Vec<RegionRelease>,
}

impl ReleaseDatesResponse {
    /// The supported-board certifications, from each region's first
    /// non-blank certification.
    fn certifications(&self) -> Certifications {
        let mut certifications = Certifications::default();
        for region in &self.results {
            if region.iso_3166_1.eq_ignore_ascii_case("US") {
                certifications.us = region.certification().map(String::from);
            } else if region.iso_3166_1.eq_ignore_ascii_case("GB") {
                certifications.gb = region.certification().map(String::from);
            } else if region.iso_3166_1.eq_ignore_ascii_case("DE") {
                certifications.de = region.certification().map(String::from);
            }
        }
        certifications
    }
}

/// One release-dates region entry.
#[derive(Debug, Deserialize)]
struct RegionRelease {
    #[serde(default)]
    iso_3166_1: String,
    /// The region's releases, in the order TMDB reports them.
    #[serde(default)]
    release_dates: Vec<ReleaseDate>,
}

impl RegionRelease {
    /// The first non-blank certification across the region's releases.
    fn certification(&self) -> Option<&str> {
        self.release_dates
            .iter()
            .find_map(|release| release.certification.as_deref().filter(|c| !c.is_empty()))
    }
}

/// One release event in a region.
#[derive(Debug, Deserialize)]
struct ReleaseDate {
    /// The board certification for this release, when known.
    #[serde(default)]
    certification: Option<String>,
}

/// TMDB's movie-detail record, converted to this crate's [`Movie`].
impl From<MovieDetails> for Movie {
    fn from(d: MovieDetails) -> Self {
        Self {
            id: i64::from(d.id.unwrap_or(0)),
            title: d.title.unwrap_or_default(),
            original_title: d.original_title.unwrap_or_default(),
            original_language: d.original_language.unwrap_or_default(),
            release_date: d.release_date,
            runtime: d.runtime,
            genres: d
                .genres
                .unwrap_or_default()
                .into_iter()
                .map(Into::into)
                .collect(),
            overview: d.overview.unwrap_or_default(),
            tagline: d.tagline,
            status: d.status.unwrap_or_default(),
            poster_path: d.poster_path,
            backdrop_path: d.backdrop_path,
            vote_average: f64::from(d.vote_average.unwrap_or(0.0)),
            vote_count: i64::from(d.vote_count.unwrap_or(0)),
            imdb_id: d.imdb_id,
            popularity: f64::from(d.popularity.unwrap_or(0.0)),
            adult: d.adult.unwrap_or(false),
            // Fetched separately in `TmdbClient::movie`.
            certifications: None,
        }
    }
}

/// TMDB's search-result movie, converted to this crate's [`MovieSummary`].
impl From<MovieObject> for MovieSummary {
    fn from(o: MovieObject) -> Self {
        Self {
            id: i64::from(o.id.unwrap_or(0)),
            title: o.title.unwrap_or_default(),
            original_title: o.original_title.unwrap_or_default(),
            original_language: o.original_language.unwrap_or_default(),
            release_date: o.release_date,
            overview: o.overview.unwrap_or_default(),
            genre_ids: o.genre_ids.unwrap_or_default(),
            poster_path: o.poster_path,
            backdrop_path: o.backdrop_path,
            vote_average: f64::from(o.vote_average.unwrap_or(0.0)),
            vote_count: i64::from(o.vote_count.unwrap_or(0)),
            popularity: f64::from(o.popularity.unwrap_or(0.0)),
            adult: o.adult.unwrap_or(false),
        }
    }
}

/// TMDB's paginated search, converted to this crate's [`SearchPage`].
impl From<MoviePaginated> for SearchPage {
    fn from(p: MoviePaginated) -> Self {
        Self {
            page: p.page.unwrap_or(0),
            results: p
                .results
                .unwrap_or_default()
                .into_iter()
                .map(Into::into)
                .collect(),
            total_pages: p.total_pages.unwrap_or(0),
            total_results: p.total_results.unwrap_or(0),
        }
    }
}

/// TMDB's genre, converted to this crate's [`Genre`].
impl From<tmdb_client::models::Genre> for Genre {
    fn from(g: tmdb_client::models::Genre) -> Self {
        Self {
            id: g.id.unwrap_or(0),
            name: g.name.unwrap_or_default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Movie, MovieDetails, MovieObject, MovieSummary};

    fn test_movie() -> Movie {
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
            poster_path: Some("/abc.jpg".into()),
            backdrop_path: Some("/def.jpg".into()),
            vote_average: 8.433,
            vote_count: 26279,
            imdb_id: Some("tt0137523".into()),
            popularity: 73.433,
            adult: false,
            certifications: None,
        }
    }

    #[test]
    fn release_year_parses() {
        let m = test_movie();
        assert_eq!(m.release_year(), Some(1999));
        let no_date = Movie {
            release_date: None,
            ..m
        };
        assert_eq!(no_date.release_year(), None);
    }

    #[test]
    fn poster_url_builds() {
        let m = test_movie();
        assert_eq!(
            m.poster_url().as_deref(),
            Some("https://image.tmdb.org/t/p/original/abc.jpg")
        );
    }

    #[test]
    fn backdrop_url_builds() {
        let m = test_movie();
        assert_eq!(
            m.backdrop_url().as_deref(),
            Some("https://image.tmdb.org/t/p/original/def.jpg")
        );
    }

    #[test]
    fn movie_details_convert() {
        let details = MovieDetails {
            id: Some(550),
            title: Some("Fight Club".into()),
            original_title: Some("Fight Club".into()),
            original_language: Some("en".into()),
            release_date: Some("1999-10-15".into()),
            imdb_id: Some("tt0137523".into()),
            ..Default::default()
        };
        let m: Movie = details.into();
        assert_eq!(m.id, 550);
        assert_eq!(m.title, "Fight Club");
        assert_eq!(m.release_date.as_deref(), Some("1999-10-15"));
        assert_eq!(m.imdb_id.as_deref(), Some("tt0137523"));
    }

    #[test]
    fn image_url_builds() {
        assert_eq!(
            super::image_url("/abc.jpg", "w342"),
            "https://image.tmdb.org/t/p/w342/abc.jpg"
        );
        assert_eq!(
            test_movie().poster_url().as_deref(),
            Some("https://image.tmdb.org/t/p/original/abc.jpg")
        );
    }

    #[test]
    fn release_dates_parse_regions() {
        let json = r#"{
            "id": 550,
            "results": [
                {
                    "iso_3166_1": "US",
                    "release_dates": [
                        { "certification": "R", "release_date": "1999-10-15T00:00:00.000Z", "type": 3 },
                        { "certification": "", "release_date": "2003-01-01T00:00:00.000Z", "type": 3 }
                    ]
                },
                { "iso_3166_1": "GB", "release_dates": [ { "certification": "15" } ] },
                { "iso_3166_1": "DE", "release_dates": [ { "certification": "FSK 16" } ] },
                { "iso_3166_1": "JP", "release_dates": [ { "certification": "R12+" } ] },
                { "iso_3166_1": "AU", "release_dates": [] }
            ]
        }"#;
        let response: super::ReleaseDatesResponse = serde_json::from_str(json).unwrap();
        let certifications = response.certifications();
        assert_eq!(certifications.us.as_deref(), Some("R"));
        assert_eq!(certifications.gb.as_deref(), Some("15"));
        assert_eq!(certifications.de.as_deref(), Some("FSK 16"));
        // Unsupported regions (JP) and release-less regions (AU) are not stored.
        assert!(!certifications.is_empty());
    }

    #[test]
    fn movie_object_to_summary() {
        let o = MovieObject {
            id: Some(550),
            title: Some("Fight Club".into()),
            genre_ids: Some(vec![18]),
            ..Default::default()
        };
        let s: MovieSummary = o.into();
        assert_eq!(s.id, 550);
        assert_eq!(s.title, "Fight Club");
        assert_eq!(s.genre_ids, vec![18]);
    }
}
