//! Integration tests for the TMDB client, served by a wiremock mock server.

use film_logger::Error;
use film_logger::tmdb::TmdbClient;
use serde_json::{Value, from_str};
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

#[tokio::test]
async fn movie_by_id() {
    let server = MockServer::builder().start().await;
    mount_json(&server, "/movie/550", 200, value(MOVIE_JSON)).await;

    let client = TmdbClient::with_base_url("test-key", server.uri());
    let movie = client.movie(550).await.unwrap();
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

#[tokio::test]
async fn find_by_imdb_id() {
    let server = MockServer::builder().start().await;
    mount_json(&server, "/find/tt0137523", 200, value(MOVIE_JSON)).await;

    let client = TmdbClient::with_base_url("test-key", server.uri());
    let movie = client.find_by_imdb_id("tt0137523").await.unwrap();
    assert_eq!(movie.id, 550);
    assert_eq!(movie.title, "Fight Club");
}

#[tokio::test]
async fn search_movies() {
    let server = MockServer::builder().start().await;
    mount_json(&server, "/search/movie", 200, value(SEARCH_JSON)).await;

    let client = TmdbClient::with_base_url("test-key", server.uri());
    let page = client.search_movies("fight club", 1).await.unwrap();
    assert_eq!(page.page, 1);
    assert_eq!(page.total_results, 1);
    assert_eq!(page.results.len(), 1);
    assert_eq!(page.results[0].id, 550);
    assert_eq!(page.results[0].title, "Fight Club");
}

#[tokio::test]
async fn api_error_surfaces_status_and_message() {
    let server = MockServer::builder().start().await;
    let body = value(
        r#"{"status_code": 34, "status_message": "The resource you requested could not be found."}"#,
    );
    mount_json(&server, "/movie/999999999", 404, body).await;

    let client = TmdbClient::with_base_url("test-key", server.uri());
    let err = client.movie(999_999_999).await.unwrap_err();
    match err {
        Error::Api { status, message } => {
            assert_eq!(status, 404);
            assert!(message.contains("could not be found"));
        }
        other => panic!("expected an Api error, got {other:?}"),
    }
}
