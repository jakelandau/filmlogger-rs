//! Throwaway acceptance check: against a mock TMDB, add films to a
//! [`FilmLogger`], export the database, and verify the exported copy reopens
//! with full parity.
//!
//! ```sh
//! cargo run --example acceptance
//! ```

use film_logger::domain::{Dci, Format, HomeMedium, Projection, ScreenClass};
use film_logger::{FilmLogger, NewLogDetails, TmdbClient, Venue, WatchLog};
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
    "status": "Released",
    "poster_path": "/abc.jpg",
    "imdb_id": "tt0137523",
    "vote_average": 8.433,
    "vote_count": 26279,
    "popularity": 73.433,
    "adult": false,
    "credits": {
        "cast": [
            { "id": 287, "name": "Brad Pitt", "character": "Tyler Durden", "order": 0 }
        ]
    },
    "release_dates": {
        "results": [
            { "iso_3166_1": "US", "release_dates": [ { "certification": "R" } ] },
            { "iso_3166_1": "GB", "release_dates": [ { "certification": "15" } ] },
            { "iso_3166_1": "DE", "release_dates": [ { "certification": "FSK 16" } ] }
        ]
    },
    "belongs_to_collection": null
}"#;

fn value(json: &str) -> Value {
    from_str(json).unwrap()
}

fn start_tmdb() -> MockServer {
    let rt = Builder::new_current_thread().enable_all().build().unwrap();
    rt.block_on(async {
        let server = MockServer::builder().start().await;
        Mock::given(path("/movie/550"))
            .respond_with(ResponseTemplate::new(200).set_body_json(value(MOVIE_JSON)))
            .mount(&server)
            .await;
        server
    })
}

fn main() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("watch_log.db");
    let exported = dir.path().join("export.db");

    let server = start_tmdb();
    let logger =
        FilmLogger::open_with(&db, TmdbClient::with_base_url("test-key", server.uri())).unwrap();

    // Two viewings of the same film: the second is a rewatch.
    let first = logger
        .add_film(
            550,
            Venue::Theatre,
            Format::Theatre {
                projection: Projection::Dci(Dci::Laser4K),
                screen: ScreenClass::Plf,
                three_d: false,
                hfr: false,
                dbox: false,
            },
            &NewLogDetails::default(),
        )
        .unwrap();
    let second = logger
        .add_film(
            550,
            Venue::Home,
            Format::Home {
                medium: HomeMedium::BluRay,
            },
            &NewLogDetails::default(),
        )
        .unwrap();
    assert_eq!(first.watch_count, Some(1));
    assert!(!first.rewatch);
    assert!(second.rewatch);
    assert_eq!(second.watch_count, Some(2));
    println!(
        "added: {} ({}), rewatch={}",
        second.title, second.watched_at, second.rewatch,
    );

    // A home viewing of a different film, added without TMDB.
    let mut home = WatchLog::now(film_logger::FilmId::Tmdb(999), "A Home Film", Venue::Home);
    home.format = Some(Format::Home {
        medium: HomeMedium::BluRay,
    });
    logger.store().insert(home).unwrap();
    println!("added: A Home Film (direct)");

    let original_list = logger.list().unwrap();
    logger.set_selected_board("DE").unwrap();
    logger.export_to(&exported).unwrap();
    println!("exported to {}", exported.display());

    // Reopen the exported file and verify parity.
    let reopened = FilmLogger::open_with(
        &exported,
        TmdbClient::with_base_url("test-key", server.uri()),
    )
    .unwrap();
    let listed = reopened.list().unwrap();
    assert_eq!(listed, original_list);
    assert_eq!(listed.len(), 3);
    assert_eq!(reopened.settings().unwrap().selected_board, "DE");
    let stats = reopened.stats().unwrap();
    assert_eq!(stats.total_viewings, 3);
    assert_eq!(stats.distinct_films, 2);
    assert_eq!(stats.rewatches, 1);

    println!(
        "stats: total={} distinct={} rewatches={} by_venue={:?} by_year={:?}",
        stats.total_viewings,
        stats.distinct_films,
        stats.rewatches,
        stats.viewings_by_venue,
        stats.viewings_by_year,
    );
    for log in &listed {
        println!(
            "  #{} {} @ {} venue={:?} rewatch={} count={:?}",
            log.id, log.title, log.watched_at, log.venue, log.rewatch, log.watch_count,
        );
    }

    println!("acceptance: OK");
}
