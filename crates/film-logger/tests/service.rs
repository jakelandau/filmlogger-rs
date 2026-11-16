//! Integration tests for the [`FilmLogger`] facade, served by a wiremock
//! mock TMDB: add films, log viewings, read the film library, export/import
//! the database, and check the aggregate breakdowns.

use film_logger::domain::{Dci, FilmId, Format, HomeMedium, Imax, Projection, ScreenClass};
use film_logger::{Error, FilmLogger, NewLogDetails, Rating, TmdbClient, Venue, WatchLog};
use serde_json::{Value, json};
use tokio::runtime::Builder;
use wiremock::matchers::path;
use wiremock::{Mock, MockServer, ResponseTemplate};

/// A TMDB movie-detail response (with the appended credits/release dates).
fn movie_json(title: &str) -> Value {
    json!({
        "id": 550,
        "title": title,
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
                { "iso_3166_1": "DE", "release_dates": [ { "certification": "FSK 16" } ] },
                { "iso_3166_1": "FR", "release_dates": [ { "certification": "U" } ] }
            ]
        },
        "belongs_to_collection": null
    })
}

fn theatre_format() -> Format {
    Format::Theatre {
        projection: Projection::Dci(Dci::Laser4K),
        screen: ScreenClass::Plf,
        three_d: false,
        hfr: false,
        dbox: false,
    }
}

fn bluray() -> Format {
    Format::Home {
        medium: HomeMedium::BluRay,
    }
}

/// Start a wiremock server whose `/movie/550` returns `title`.
fn start_tmdb(title: &str) -> MockServer {
    let body = movie_json(title);
    let rt = Builder::new_current_thread().enable_all().build().unwrap();
    rt.block_on(async {
        let server = MockServer::builder().start().await;
        Mock::given(path("/movie/550"))
            .respond_with(ResponseTemplate::new(200).set_body_json(body))
            .mount(&server)
            .await;
        server
    })
}

fn logger_at(path: &std::path::Path, server: &MockServer) -> FilmLogger {
    FilmLogger::open_with(path, TmdbClient::with_base_url("test-key", server.uri())).unwrap()
}

#[test]
fn add_export_reopen_parity() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("watch_log.db");
    let exported = dir.path().join("export.db");

    let server = start_tmdb("Fight Club");
    let logger = logger_at(&db, &server);

    // Add a film: enriched from TMDB, with certifications.
    let log = logger
        .add_film(
            550,
            Venue::Theatre,
            theatre_format(),
            &NewLogDetails::default(),
        )
        .unwrap();
    assert_eq!(log.title, "Fight Club");
    assert_eq!(log.format_summary, "4K laser · PLF");
    let certifications = log.certifications.as_ref().unwrap();
    assert_eq!(certifications.get("US"), Some("R"));
    assert_eq!(certifications.get("GB"), Some("15"));
    assert_eq!(certifications.get("DE"), Some("FSK 16"));

    // The film is in the library with its viewing summary.
    let films = logger.films().unwrap();
    assert_eq!(films.len(), 1);
    assert_eq!(films[0].film.title, "Fight Club");
    assert_eq!(films[0].viewings, 1);

    logger.set_selected_board("GB").unwrap();
    logger.export_to(&exported).unwrap();

    // Reopen the exported file: full data and settings parity.
    let reopened = logger_at(&exported, &server);
    assert_eq!(reopened.list().unwrap(), vec![log]);
    assert_eq!(reopened.settings().unwrap().selected_board, "GB");
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

    let server = start_tmdb("Fight Club");

    // The source database holds one film; the target a different one.
    let source = logger_at(&source_db, &server);
    source
        .add_film(
            550,
            Venue::Theatre,
            theatre_format(),
            &NewLogDetails::default(),
        )
        .unwrap();
    source.export_to(&source_export).unwrap();

    let mut target = logger_at(&target_db, &server);
    let mut home = WatchLog::now(FilmId::Tmdb(999), "A Home Film", Venue::Home);
    home.format = Some(bluray());
    target.store().insert(home).unwrap();

    // Import: the target's contents become the source's, including settings.
    target.set_selected_board("US").unwrap();
    target.import_from(&source_export).unwrap();
    assert_eq!(target.settings().unwrap().selected_board, "US");
    let listed = target.list().unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].title, "Fight Club");
}

#[test]
fn add_film_applies_details() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("watch_log.db");
    let server = start_tmdb("Fight Club");
    let logger = logger_at(&db, &server);

    let watched_at = chrono::DateTime::parse_from_rfc3339("2020-06-01T12:00:00Z")
        .unwrap()
        .with_timezone(&chrono::Utc);
    let details = NewLogDetails {
        watched_at: Some(watched_at),
        timezone: Some(" Europe/London ".into()),
        notes: Some("  great  ".into()),
        rating: Some(Rating::Up),
    };
    let log = logger
        .add_film(550, Venue::Theatre, theatre_format(), &details)
        .unwrap();
    assert_eq!(log.watched_at, watched_at);
    assert_eq!(log.timezone.as_deref(), Some("Europe/London"));
    assert_eq!(log.notes.as_deref(), Some("great"));
    assert_eq!(log.rating, Some(Rating::Up));
    assert_eq!(log.format_summary, "4K laser · PLF");
}

#[test]
fn log_film_uses_stored_metadata_without_network() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("watch_log.db");
    let server = start_tmdb("Fight Club");
    let logger = logger_at(&db, &server);
    let first = logger
        .add_film(
            550,
            Venue::Theatre,
            theatre_format(),
            &NewLogDetails::default(),
        )
        .unwrap();
    logger.delete(first.id).unwrap();

    // A client pointed at a port nothing listens on: `log_film` must not use
    // the network.
    let offline = FilmLogger::open_with(
        &db,
        TmdbClient::with_base_url("test-key", "http://127.0.0.1:1"),
    )
    .unwrap();
    let log = offline
        .log_film(
            &FilmId::Imdb("tt0137523".into()),
            Venue::Home,
            bluray(),
            &NewLogDetails::default(),
        )
        .unwrap();
    assert_eq!(log.title, "Fight Club");
    assert_eq!(log.poster_path.as_deref(), Some("/abc.jpg"));
    assert_eq!(log.certifications.as_ref().unwrap().get("US"), Some("R"));
    assert_eq!(log.watch_count, Some(1));
    assert_eq!(log.format_summary, "Blu-ray");
}

#[test]
fn refresh_film_preserves_user_state() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("watch_log.db");
    let server = start_tmdb("Fight Club");
    let logger = logger_at(&db, &server);
    logger
        .add_film(
            550,
            Venue::Theatre,
            theatre_format(),
            &NewLogDetails::default(),
        )
        .unwrap();
    let film_id = FilmId::Imdb("tt0137523".into());
    logger.set_favorite(&film_id, true).unwrap();

    // A second server reports a new title.
    let changed = start_tmdb("Fight Club (Remastered)");
    let refreshed = logger_at(&db, &changed).refresh_film(&film_id).unwrap();
    assert_eq!(refreshed.title, "Fight Club (Remastered)");
    assert!(refreshed.favorite);
    assert!(refreshed.metadata_refreshed_at.is_some());
}

#[test]
fn film_page_reports_history_and_stats() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("watch_log.db");
    let server = start_tmdb("Fight Club");
    let logger = logger_at(&db, &server);
    logger
        .add_film(
            550,
            Venue::Theatre,
            theatre_format(),
            &NewLogDetails::default(),
        )
        .unwrap();
    let film_id = FilmId::Imdb("tt0137523".into());
    let earlier = chrono::DateTime::parse_from_rfc3339("2020-06-01T12:00:00Z")
        .unwrap()
        .with_timezone(&chrono::Utc);
    logger
        .log_film(
            &film_id,
            Venue::Home,
            bluray(),
            &NewLogDetails {
                watched_at: Some(earlier),
                ..NewLogDetails::default()
            },
        )
        .unwrap();

    let page = logger.film_page(&film_id).unwrap();
    assert_eq!(page.film.title, "Fight Club");
    assert_eq!(page.logs.len(), 2);
    assert!(page.logs[0].watched_at > page.logs[1].watched_at);
    assert_eq!(page.stats.viewings, 2);
    assert_eq!(page.stats.theatre, 1);
    assert_eq!(page.stats.home, 1);
    assert_eq!(page.stats.first_watched_at, Some(earlier));
    assert_eq!(page.stats.last_watched_at, Some(page.logs[0].watched_at));
    assert_eq!(page.stats.formats.get("4K laser · PLF").copied(), Some(1));
    assert_eq!(page.stats.formats.get("Blu-ray").copied(), Some(1));
}

#[test]
fn stats_breakdowns() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("watch_log.db");
    let server = start_tmdb("Fight Club");
    let logger = logger_at(&db, &server);

    let watched_at = chrono::DateTime::parse_from_rfc3339("2021-03-04T20:00:00Z")
        .unwrap()
        .with_timezone(&chrono::Utc);
    let store = logger.store();
    let mut inserts = Vec::new();
    for (venue, format) in [
        (
            Venue::Theatre,
            Format::Theatre {
                projection: Projection::Dci(Dci::Laser4K),
                screen: ScreenClass::Plf,
                three_d: true,
                hfr: false,
                dbox: false,
            },
        ),
        (Venue::Home, bluray()),
        (
            Venue::Theatre,
            Format::Theatre {
                projection: Projection::Imax(Imax::Laser4KDual),
                screen: ScreenClass::ImaxClassic,
                three_d: true,
                hfr: false,
                dbox: false,
            },
        ),
    ] {
        let mut log = WatchLog::now(FilmId::Imdb("tt0137523".into()), "Fight Club", venue);
        log.watched_at = watched_at;
        log.format = Some(format);
        inserts.push(store.insert(log).unwrap());
    }
    assert_eq!(inserts.len(), 3);

    let stats = logger.stats().unwrap();
    assert_eq!(stats.viewings_by_projection_family.get("dci"), Some(&1));
    assert_eq!(stats.viewings_by_projection_family.get("imax"), Some(&1));
    assert_eq!(stats.viewings_by_projection_family.get("home"), Some(&1));
    assert_eq!(stats.viewings_by_screen.get("plf"), Some(&1));
    assert_eq!(stats.viewings_by_screen.get("imax_class"), Some(&1));
    assert_eq!(stats.viewings_by_medium.get("bluray"), Some(&1));
    assert_eq!(stats.viewings_by_flag.get("three_d"), Some(&2));
    assert_eq!(stats.viewings_by_month.get("2021-03"), Some(&3));
}

#[test]
fn log_film_rejects_unknown_film() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("watch_log.db");
    let server = start_tmdb("Fight Club");
    let logger = logger_at(&db, &server);
    let err = logger
        .log_film(
            &FilmId::Imdb("tt9999999".into()),
            Venue::Home,
            bluray(),
            &NewLogDetails::default(),
        )
        .unwrap_err();
    assert!(matches!(err, Error::FilmNotFound));
}

#[test]
fn add_to_watchlist_stores_without_a_viewing() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("watch_log.db");
    let server = start_tmdb("Fight Club");
    let logger = logger_at(&db, &server);

    let film = logger.add_to_watchlist(550).unwrap();
    assert!(film.watchlisted);
    assert_eq!(film.title, "Fight Club");

    let films = logger.films().unwrap();
    assert_eq!(films.len(), 1);
    assert_eq!(films[0].viewings, 0);
    assert!(films[0].film.watchlisted);
    assert_eq!(films[0].film.film_id, FilmId::Imdb("tt0137523".to_owned()));
}
