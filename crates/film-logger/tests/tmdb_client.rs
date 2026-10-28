//! Integration tests for the TMDB client, served by a wiremock mock server.
//!
//! The [`TmdbClient`] wraps a blocking (synchronous) TMDB client, so the tests
//! are synchronous: the async [`MockServer`] setup is driven on a throwaway
//! tokio runtime, after which the client calls block on the current thread.

use film_logger::Error;
use film_logger::tmdb::TmdbClient;
use serde_json::{Value, from_str};
use tokio::runtime::Builder;
use wiremock::matchers::path;
use wiremock::{Mock, MockServer, ResponseTemplate};

const MOVIE_JSON: &str = r#"{
    "id": 550,
    "title": "Fight Club",
    "original_title": "Fight Club",
    "original_language": "en",
    "release_date": "1999-10-15",
    "runtime": 139,
    "genres": [{ "id": 18, "name": "Drama" }],
    "overview": "A story about two men.",
    "tagline": null,
    "status": "Released",
    "poster_path": "/abc.jpg",
    "backdrop_path": "/def.jpg",
    "vote_average": 8.433,
    "vote_count": 26279,
    "imdb_id": "tt0137523",
    "popularity": 73.433,
    "adult": false
}"#;

const FIND_JSON: &str = r#"{
    "movie_results": [
        {
            "id": 550,
            "title": "Fight Club",
            "original_title": "Fight Club",
            "original_language": "en",
            "release_date": "1999-10-15",
            "overview": "A story about two men.",
            "genre_ids": [18],
            "poster_path": "/abc.jpg",
            "backdrop_path": "/def.jpg",
            "vote_average": 8.433,
            "vote_count": 26279,
            "popularity": 73.433,
            "adult": false
        }
    ],
    "person_results": [],
    "tv_results": [],
    "tv_episode_results": [],
    "tv_season_results": []
}"#;

const SEARCH_JSON: &str = r#"{
    "page": 1,
    "results": [
        {
            "id": 550,
            "title": "Fight Club",
            "original_title": "Fight Club",
            "original_language": "en",
            "release_date": "1999-10-15",
            "overview": "A story about two men.",
            "genre_ids": [18],
            "poster_path": "/abc.jpg",
            "backdrop_path": "/def.jpg",
            "vote_average": 8.433,
            "vote_count": 26279,
            "popularity": 73.433,
            "adult": false
        }
    ],
    "total_pages": 1,
    "total_results": 1
}"#;

fn value(json: &str) -> Value {
    from_str(json).unwrap()
}

/// Mount a JSON responder for a path on the server, expected exactly once.
async fn mount_json(server: &MockServer, p: &str, status: u16, body: Value) {
    Mock::given(path(p))
        .respond_with(ResponseTemplate::new(status).set_body_json(body))
        .expect(1)
        .mount(server)
        .await;
}

/// Start a wiremock [`MockServer`] and mount the given JSON responders. The
/// async setup is driven on a throwaway single-threaded tokio runtime; the
/// server runs on its own thread once started, so the runtime only needs to
/// live long enough to complete setup.
fn start_server(mounts: Vec<(&str, u16, Value)>) -> MockServer {
    let rt = Builder::new_current_thread().enable_all().build().unwrap();
    rt.block_on(async move {
        let server = MockServer::builder().start().await;
        for (p, status, body) in mounts {
            mount_json(&server, p, status, body).await;
        }
        server
    })
}

#[test]
fn movie_by_id() {
    let server = start_server(vec![("/movie/550", 200, value(MOVIE_JSON))]);
    let client = TmdbClient::with_base_url("test-key", server.uri());
    let movie = client.movie(550).unwrap();
    assert_eq!(movie.id, 550);
    assert_eq!(movie.title, "Fight Club");
    assert_eq!(movie.imdb_id.as_deref(), Some("tt0137523"));
    assert_eq!(movie.release_year(), Some(1999));
    assert_eq!(
        movie.poster_url().as_deref(),
        Some("https://image.tmdb.org/t/p/original/abc.jpg")
    );
    assert_eq!(movie.genres.len(), 1);
    assert_eq!(movie.genres[0].name, "Drama");
}

#[test]
fn find_by_imdb_id() {
    let server = start_server(vec![
        ("/find/tt0137523", 200, value(FIND_JSON)),
        ("/movie/550", 200, value(MOVIE_JSON)),
    ]);
    let client = TmdbClient::with_base_url("test-key", server.uri());
    let movie = client.find_by_imdb_id("tt0137523").unwrap();
    assert_eq!(movie.id, 550);
    assert_eq!(movie.title, "Fight Club");
    assert_eq!(movie.release_year(), Some(1999));
}

#[test]
fn find_by_imdb_id_not_found() {
    let server = start_server(vec![(
        "/find/tt9999999",
        200,
        value("{\"movie_results\":[]}"),
    )]);
    let client = TmdbClient::with_base_url("test-key", server.uri());
    let err = client.find_by_imdb_id("tt9999999").unwrap_err();
    assert!(matches!(err, Error::FilmNotFound));
}

#[test]
fn search_movies() {
    let server = start_server(vec![("/search/movie", 200, value(SEARCH_JSON))]);
    let client = TmdbClient::with_base_url("test-key", server.uri());
    let page = client.search_movies("fight club", 1).unwrap();
    assert_eq!(page.page, 1);
    assert_eq!(page.total_results, 1);
    assert_eq!(page.results.len(), 1);
    assert_eq!(page.results[0].id, 550);
    assert_eq!(page.results[0].title, "Fight Club");
}

#[test]
fn api_error_surfaces_status_and_message() {
    let body = value(
        r#"{"status_code": 34, "status_message": "The resource you requested could not be found."}"#,
    );
    let server = start_server(vec![("/movie/999999999", 404, body)]);
    let client = TmdbClient::with_base_url("test-key", server.uri());
    let err = client.movie(999_999_999).unwrap_err();
    match err {
        Error::Api { status, message } => {
            assert_eq!(status, 404);
            assert_ne!(message, "");
        }
        other => panic!("expected an Api error, got {other:?}"),
    }
}
