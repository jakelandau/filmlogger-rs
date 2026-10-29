//! Integration tests for the [`FilmLogger`] facade, served by a wiremock
//! mock TMDB: add a film, export the database, and verify the exported copy
//! reopens with full data and settings parity.

use film_logger::{FilmLogger, RatingBoard, TmdbClient, Venue};
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

const RELEASE_DATES_JSON: &str = r#"{
    "id": 550,
    "results": [
        { "iso_3166_1": "US", "release_dates": [ { "certification": "R", "release_date": "1999-10-15T00:00:00.000Z", "type": 3 } ] },
        { "iso_3166_1": "GB", "release_dates": [ { "certification": "15" } ] },
        { "iso_3166_1": "DE", "release_dates": [ { "certification": "FSK 16" } ] },
        { "iso_3166_1": "FR", "release_dates": [ { "certification": "U" } ] }
    ]
}"#;

fn value(json: &str) -> Value {
    from_str(json).unwrap()
}

/// Start a wiremock server answering the movie-details and release-dates
/// endpoints for TMDB id 550. The async setup is driven on a throwaway
/// single-threaded tokio runtime; the server then runs on its own thread.
fn start_tmdb() -> MockServer {
    let rt = Builder::new_current_thread().enable_all().build().unwrap();
    rt.block_on(async {
        let server = MockServer::builder().start().await;
        // Details first, release dates second: the latter's path contains the
        // former's, so the later mount must take precedence on the overlap.
        Mock::given(path("/movie/550"))
            .respond_with(ResponseTemplate::new(200).set_body_json(value(MOVIE_JSON)))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(path("/movie/550/release_dates"))
            .respond_with(ResponseTemplate::new(200).set_body_json(value(RELEASE_DATES_JSON)))
            .expect(1)
            .mount(&server)
            .await;
        server
    })
}

#[test]
fn add_export_reopen_parity() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("watch_log.db");
    let exported = dir.path().join("export.db");

    let server = start_tmdb();
    let logger =
        FilmLogger::open_with(&db, TmdbClient::with_base_url("test-key", server.uri())).unwrap();

    // Add a film: enriched from TMDB, with certifications.
    let log = logger.add_film(550, Venue::Theatre, None).unwrap();
    assert_eq!(log.title, "Fight Club");
    let certifications = log.certifications.as_ref().unwrap();
    assert_eq!(certifications.us.as_deref(), Some("R"));
    assert_eq!(certifications.gb.as_deref(), Some("15"));
    assert_eq!(certifications.de.as_deref(), Some("FSK 16"));

    logger.set_selected_board(RatingBoard::Bbfc).unwrap();
    logger.export_to(&exported).unwrap();

    // Reopen the exported file: full data and settings parity.
    let reopened = FilmLogger::open_with(
        &exported,
        TmdbClient::with_base_url("test-key", server.uri()),
    )
    .unwrap();
    let listed = reopened.list().unwrap();
    assert_eq!(listed, vec![log]);
    assert_eq!(
        reopened.settings().unwrap().selected_board,
        RatingBoard::Bbfc
    );
    let stats = reopened.stats().unwrap();
    assert_eq!(stats.total_viewings, 1);
    assert_eq!(stats.distinct_films, 1);
}

#[test]
fn import_from_replaces_contents() {
    let dir = tempfile::tempdir().unwrap();
    let source_db = dir.path().join("source.db");
    let source_export = dir.path().join("source_export.db");
    let target_db = dir.path().join("target.db");

    let server = start_tmdb();

    // The source database holds one film; the target a different one.
    let source = FilmLogger::open_with(
        &source_db,
        TmdbClient::with_base_url("test-key", server.uri()),
    )
    .unwrap();
    source.add_film(550, Venue::Theatre, None).unwrap();
    source.export_to(&source_export).unwrap();

    let mut target = FilmLogger::open_with(
        &target_db,
        TmdbClient::with_base_url("test-key", server.uri()),
    )
    .unwrap();
    let home =
        film_logger::WatchLog::now(film_logger::FilmId::Tmdb(999), "A Home Film", Venue::Home);
    target.store().insert(home).unwrap();

    // Import: the target's contents become the source's, including settings.
    target.set_selected_board(RatingBoard::Mpa).unwrap();
    target.import_from(&source_export).unwrap();
    assert_eq!(target.settings().unwrap().selected_board, RatingBoard::Mpa);
    let listed = target.list().unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].title, "Fight Club");
}
