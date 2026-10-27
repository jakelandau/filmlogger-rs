//! TMDB API client and models.
//!
//! The client is async (reqwest) and platform-agnostic. It speaks the TMDB
//! v3 REST API: fetching a full movie by TMDB id, finding a movie by IMDB id,
//! and searching movies. Images are referenced by path; the full image URLs
//! are built by [`Movie::poster_url`] and [`Movie::backdrop_url`].

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

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
            .map(|p| format!("{IMAGE_BASE_URL}/original{p}"))
    }

    /// The full URL for the backdrop image, if a backdrop path is known.
    #[must_use]
    pub fn backdrop_url(&self) -> Option<String> {
        self.backdrop_path
            .as_deref()
            .map(|p| format!("{IMAGE_BASE_URL}/original{p}"))
    }
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
pub struct TmdbClient {
    http: reqwest::Client,
    /// True when the key is a JWT (a TMDB API Read Access Token) and should
    /// be sent as a `Bearer` token; false when it is the legacy API Key
    /// (sent as the `?api_key=` query parameter).
    is_jwt: bool,
    api_key: String,
    language: String,
    base_url: String,
}

impl TmdbClient {
    /// Create a client for the live TMDB API.
    ///
    /// `api_key` may be either the TMDB **API Read Access Token** (a JWT,
    /// beginning with `eyJ`) or the legacy **API Key**; the client detects
    /// which and sends it accordingly (Bearer token vs `?api_key=`).
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
        Self {
            http: reqwest::Client::new(),
            is_jwt: api_key.starts_with("eyJ"),
            api_key,
            language: "en".into(),
            base_url: base_url.into(),
        }
    }

    /// Get a full movie by its TMDB numeric id.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Api`] on an HTTP error status (e.g. 404 not found),
    /// [`Error::Json`] if the response body is not valid JSON,
    /// [`Error::Transport`] on connection or timeout errors, or
    /// [`Error::Http`] for other transport errors.
    pub async fn movie(&self, id: i64) -> Result<Movie, Error> {
        let path = format!("/movie/{id}");
        self.get(&path, &[]).await
    }

    /// Find a movie by its IMDB id (e.g. `tt0137523`).
    ///
    /// # Errors
    ///
    /// Returns [`Error::Api`] on an HTTP error status, [`Error::Json`] if the
    /// response body is not valid JSON, [`Error::Transport`] on connection or
    /// timeout errors, or [`Error::Http`] for other transport errors.
    pub async fn find_by_imdb_id(&self, imdb_id: &str) -> Result<Movie, Error> {
        let path = format!("/find/{imdb_id}");
        self.get(&path, &[("type", "movie")]).await
    }

    /// Search movies by query.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Api`] on an HTTP error status, [`Error::Json`] if the
    /// response body is not valid JSON, [`Error::Transport`] on connection or
    /// timeout errors, or [`Error::Http`] for other transport errors.
    pub async fn search_movies(&self, query: &str, page: i32) -> Result<SearchPage, Error> {
        let page = page.to_string();
        self.get("/search/movie", &[("query", query), ("page", &page)])
            .await
    }

    async fn get<T: DeserializeOwned>(
        &self,
        path: &str,
        extra: &[(&str, &str)],
    ) -> Result<T, Error> {
        let mut params: Vec<(&str, &str)> = vec![("language", self.language.as_str())];
        if !self.is_jwt {
            params.push(("api_key", self.api_key.as_str()));
        }
        params.extend(extra.iter().copied());
        let url = format!("{}{path}", self.base_url);
        let mut req = self.http.get(url).query(&params);
        if self.is_jwt {
            req = req.header(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {}", self.api_key),
            );
        }
        let resp = req.send().await?;
        let status = resp.status();
        if status.is_success() {
            let value: T = resp.json().await?;
            Ok(value)
        } else {
            let message = resp
                .json::<ApiError>()
                .await
                .ok()
                .and_then(|e| e.status_message)
                .unwrap_or_else(|| status.canonical_reason().unwrap_or("error").to_string());
            Err(Error::Api {
                status: status.as_u16(),
                message,
            })
        }
    }
}

/// TMDB's error body (for non-success responses).
#[derive(Deserialize)]
struct ApiError {
    #[serde(default)]
    status_message: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::{Movie, TmdbClient};

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
    fn jwt_uses_bearer_api_key_uses_query() {
        // A JWT (TMDB API Read Access Token) is sent as a Bearer token.
        assert!(TmdbClient::new("eyJhbGciOiJIUzI1NiJ9.abc.def").is_jwt);
        // A legacy API Key (not a JWT) is sent as ?api_key=.
        assert!(!TmdbClient::new("abcdef1234567890").is_jwt);
    }
}
