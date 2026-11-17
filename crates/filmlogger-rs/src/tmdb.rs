//! TMDB API client and models.
//!
//! A direct, blocking [`reqwest`] client for the TMDB v3 REST API. It fetches
//! a full movie by TMDB id (with credits and release dates appended), finds a
//! movie by IMDB id, searches movies, and lists the popular ones. Images are
//! referenced by path; the full image URLs are built by [`Movie::poster_url`]
//! and [`Movie::backdrop_url`].
//!
//! Authentication follows the key's shape: a v4 **API Read Access Token** (a
//! JWT, starting with `eyJ`) is sent as `Authorization: Bearer`, anything else
//! as the `?api_key=` query parameter.

use std::time::Duration;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::domain::{CastMember, Certifications, Collection, CollectionPart, CollectionRef, Genre};
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
    /// The top-billed cast (up to ten).
    pub cast: Vec<CastMember>,
    /// The collection the film belongs to, if any.
    pub collection: Option<CollectionRef>,
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

/// How requests authenticate: a v4 bearer token or a v3 API key.
enum Auth {
    /// A v3 key, sent as `?api_key=`.
    ApiKey(String),
    /// A v4 Read Access Token, sent as `Authorization: Bearer`.
    Bearer(String),
}

/// The TMDB API client.
///
/// Blocking (synchronous): the apps call it from a worker thread. The struct
/// is naturally `Send + Sync`, so it can be shared behind a mutex.
pub struct TmdbClient {
    http: reqwest::blocking::Client,
    base_url: String,
    auth: Auth,
}

impl TmdbClient {
    /// Create a client for the live TMDB API.
    ///
    /// `api_key` may be either a v3 API key or a v4 Read Access Token; the
    /// shape of the string selects the authentication scheme.
    #[must_use]
    pub fn new(api_key: impl Into<String>) -> Self {
        Self::make(api_key, TMDB_BASE_URL)
    }

    /// Create a client against a custom base URL (for tests).
    #[must_use]
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
        let auth = if api_key.starts_with("eyJ") {
            Auth::Bearer(api_key)
        } else {
            Auth::ApiKey(api_key)
        };
        let http = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(15))
            .user_agent(concat!("FilmLogger/", env!("CARGO_PKG_VERSION")))
            .build()
            .unwrap_or_else(|_| reqwest::blocking::Client::new());
        Self {
            http,
            base_url: base_url.into(),
            auth,
        }
    }

    /// Send a `GET` request and deserialize the JSON body.
    fn get<T: DeserializeOwned>(&self, path: &str, query: &[(&str, String)]) -> Result<T, Error> {
        let mut request = self
            .http
            .get(format!("{}{path}", self.base_url))
            .query(query);
        request = match &self.auth {
            Auth::ApiKey(key) => request.query(&[("api_key", key.as_str())]),
            Auth::Bearer(token) => request.bearer_auth(token),
        };
        let response = request.send().map_err(Error::Http)?;
        let status = response.status();
        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(Error::FilmNotFound);
        }
        let bytes = response.bytes().map_err(Error::Http)?;
        if !status.is_success() {
            let message = serde_json::from_slice::<ApiErrorBody>(&bytes)
                .ok()
                .and_then(|body| body.status_message)
                .filter(|m| !m.is_empty())
                .or_else(|| status.canonical_reason().map(str::to_owned))
                .unwrap_or_else(|| format!("HTTP {status}"));
            return Err(Error::Api {
                status: status.as_u16(),
                message,
            });
        }
        Ok(serde_json::from_slice(&bytes)?)
    }

    /// Get a full movie by its TMDB numeric id, including credits and board
    /// certifications.
    ///
    /// # Errors
    ///
    /// Returns [`Error::FilmNotFound`] on a 404, [`Error::Api`] on another
    /// HTTP error status, [`Error::Serde`] if the response body is not valid
    /// JSON, or [`Error::Http`] on connection, timeout, or other transport
    /// errors.
    pub fn movie(&self, id: i64) -> Result<Movie, Error> {
        let response: MovieResponse = self.get(
            &format!("/movie/{id}"),
            &[
                ("language", "en".to_owned()),
                ("append_to_response", "credits,release_dates".to_owned()),
            ],
        )?;
        Ok(response.into_movie())
    }

    /// Get a TMDB collection and its films.
    ///
    /// # Errors
    ///
    /// Returns [`Error::FilmNotFound`] on a 404, [`Error::Api`] on another
    /// HTTP error status, [`Error::Serde`] if the response body is not valid
    /// JSON, or [`Error::Http`] on connection, timeout, or other transport
    /// errors.
    pub fn collection(&self, id: i64) -> Result<Collection, Error> {
        let response: CollectionDetailsResponse = self.get(
            &format!("/collection/{id}"),
            &[("language", "en".to_owned())],
        )?;
        Ok(response.into_collection())
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
        let find: FindResponse = self.get(
            &format!("/find/{imdb_id}"),
            &[
                ("external_source", "imdb_id".to_owned()),
                ("language", "en".to_owned()),
            ],
        )?;
        let id = find
            .movie_results
            .first()
            .map(|m| m.id)
            .ok_or(Error::FilmNotFound)?;
        self.movie(id)
    }

    /// Search movies by query.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Api`] on an HTTP error status, [`Error::Serde`] if
    /// the response body is not valid JSON, or [`Error::Http`] on connection,
    /// timeout, or other transport errors.
    pub fn search_movies(&self, query: &str, page: i32) -> Result<SearchPage, Error> {
        let response: SearchResponse = self.get(
            "/search/movie",
            &[
                ("query", query.to_owned()),
                ("language", "en".to_owned()),
                ("page", page.to_string()),
                ("include_adult", "false".to_owned()),
            ],
        )?;
        Ok(response.into_page())
    }

    /// The currently popular movies.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Api`] on an HTTP error status, [`Error::Serde`] if
    /// the response body is not valid JSON, or [`Error::Http`] on connection,
    /// timeout, or other transport errors.
    pub fn popular_movies(&self, page: i32) -> Result<SearchPage, Error> {
        let response: SearchResponse = self.get(
            "/movie/popular",
            &[("language", "en".to_owned()), ("page", page.to_string())],
        )?;
        Ok(response.into_page())
    }
}

/// TMDB's error body.
#[derive(Debug, Deserialize)]
struct ApiErrorBody {
    #[serde(default)]
    status_message: Option<String>,
}

/// TMDB's movie-detail response (with appended `credits` and `release_dates`).
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct MovieResponse {
    id: Option<i64>,
    title: Option<String>,
    original_title: Option<String>,
    original_language: Option<String>,
    release_date: Option<String>,
    runtime: Option<i32>,
    genres: Vec<GenreResponse>,
    overview: Option<String>,
    tagline: Option<String>,
    status: Option<String>,
    poster_path: Option<String>,
    backdrop_path: Option<String>,
    vote_average: Option<f64>,
    vote_count: Option<i64>,
    imdb_id: Option<String>,
    popularity: Option<f64>,
    adult: Option<bool>,
    credits: Option<CreditsResponse>,
    release_dates: Option<ReleaseDatesResponse>,
    belongs_to_collection: Option<CollectionResponse>,
}

impl MovieResponse {
    /// Convert into this crate's [`Movie`], keeping the top ten cast members.
    fn into_movie(self) -> Movie {
        let cast = self
            .credits
            .map(|credits| {
                credits
                    .cast
                    .into_iter()
                    .take(10)
                    .map(CastResponse::into_member)
                    .collect()
            })
            .unwrap_or_default();
        Movie {
            id: self.id.unwrap_or(0),
            title: self.title.unwrap_or_default(),
            original_title: self.original_title.unwrap_or_default(),
            original_language: self.original_language.unwrap_or_default(),
            release_date: self.release_date,
            runtime: self.runtime,
            genres: self
                .genres
                .into_iter()
                .map(GenreResponse::into_genre)
                .collect(),
            overview: self.overview.unwrap_or_default(),
            tagline: self.tagline,
            status: self.status.unwrap_or_default(),
            poster_path: self.poster_path,
            backdrop_path: self.backdrop_path,
            vote_average: self.vote_average.unwrap_or(0.0),
            vote_count: self.vote_count.unwrap_or(0),
            imdb_id: self.imdb_id,
            popularity: self.popularity.unwrap_or(0.0),
            adult: self.adult.unwrap_or(false),
            certifications: self.release_dates.map(|r| r.certifications()),
            cast,
            collection: self.belongs_to_collection.map(CollectionResponse::into_ref),
        }
    }
}

/// TMDB's genre entry.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct GenreResponse {
    id: i32,
    name: String,
}

impl GenreResponse {
    fn into_genre(self) -> Genre {
        Genre {
            id: self.id,
            name: self.name,
        }
    }
}

/// TMDB's `credits` block.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct CreditsResponse {
    cast: Vec<CastResponse>,
}

/// TMDB's cast entry.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct CastResponse {
    id: i64,
    name: String,
    character: Option<String>,
    order: i32,
}

impl CastResponse {
    fn into_member(self) -> CastMember {
        CastMember {
            id: self.id,
            name: self.name,
            character: self.character,
            order: self.order,
        }
    }
}

/// TMDB's collection entry.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct CollectionResponse {
    id: i64,
    name: String,
    poster_path: Option<String>,
}

impl CollectionResponse {
    fn into_ref(self) -> CollectionRef {
        CollectionRef {
            id: self.id,
            name: self.name,
            poster_path: self.poster_path,
        }
    }
}

/// TMDB's collection-detail response.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct CollectionDetailsResponse {
    id: i64,
    name: String,
    overview: Option<String>,
    poster_path: Option<String>,
    backdrop_path: Option<String>,
    parts: Vec<CollectionPartResponse>,
}

impl CollectionDetailsResponse {
    fn into_collection(self) -> Collection {
        Collection {
            id: self.id,
            name: self.name,
            overview: self.overview,
            poster_path: self.poster_path,
            backdrop_path: self.backdrop_path,
            parts: self
                .parts
                .into_iter()
                .map(CollectionPartResponse::into_part)
                .collect(),
        }
    }
}

/// TMDB's collection-part entry.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct CollectionPartResponse {
    id: i64,
    title: String,
    release_date: Option<String>,
    poster_path: Option<String>,
}

impl CollectionPartResponse {
    fn into_part(self) -> CollectionPart {
        let release_year = self
            .release_date
            .as_deref()
            .and_then(|d| d.get(..4))
            .and_then(|s| s.parse::<i32>().ok());
        CollectionPart {
            tmdb_id: self.id,
            title: self.title,
            release_date: self.release_date,
            release_year,
            poster_path: self.poster_path,
        }
    }
}

/// TMDB's paginated search/popular response.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct SearchResponse {
    page: i32,
    results: Vec<MovieSummary>,
    total_pages: i32,
    total_results: i32,
}

impl SearchResponse {
    fn into_page(self) -> SearchPage {
        SearchPage {
            page: self.page,
            results: self.results,
            total_pages: self.total_pages,
            total_results: self.total_results,
        }
    }
}

/// TMDB's `find` response.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct FindResponse {
    movie_results: Vec<FindMovie>,
}

/// One movie result of a `find` lookup.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct FindMovie {
    id: i64,
}

/// TMDB's release-dates block: per-region entries whose releases carry board
/// certifications.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct ReleaseDatesResponse {
    results: Vec<RegionRelease>,
}

impl ReleaseDatesResponse {
    /// Every region's first non-blank certification.
    fn certifications(&self) -> Certifications {
        let mut certifications = Certifications::default();
        for region in &self.results {
            if let Some(certification) = region.certification() {
                certifications.insert(region.iso_3166_1.clone(), certification);
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

#[cfg(test)]
mod tests {
    use super::Movie;

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
            cast: vec![],
            collection: None,
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
    fn image_url_builds() {
        assert_eq!(
            super::image_url("/abc.jpg", "w342"),
            "https://image.tmdb.org/t/p/w342/abc.jpg"
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
        assert_eq!(certifications.get("US"), Some("R"));
        assert_eq!(certifications.get("GB"), Some("15"));
        assert_eq!(certifications.get("DE"), Some("FSK 16"));
        assert_eq!(certifications.get("JP"), Some("R12+"));
        // Release-less regions (AU) are not stored.
        assert_eq!(certifications.get("AU"), None);
        assert!(!certifications.is_empty());
    }
}
