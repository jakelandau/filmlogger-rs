//! Integration tests for the SQLite store, using temp files where
//! cross-instance behaviour (migration, export/import) is under test.

use film_logger::domain::{Dci, FilmId, Format, HomeMedium, Projection, ScreenClass};
use film_logger::{Error, FilmMetadata, Store, Venue, WatchLog};
use rusqlite::Connection;

/// A valid log for tests: a [`WatchLog::now`] with a format attached,
/// matching the venue.
fn test_log(film_id: FilmId, title: &str, venue: Venue) -> WatchLog {
    let mut log = WatchLog::now(film_id, title, venue);
    let format = if venue == Venue::Home {
        Format::Home {
            medium: HomeMedium::BluRay,
        }
    } else {
        Format::Theatre {
            projection: Projection::Dci(Dci::Laser4K),
            screen: ScreenClass::Plf,
            three_d: false,
            hfr: false,
            dbox: false,
        }
    };
    log.format = Some(format);
    log
}

fn at(iso: &str) -> chrono::DateTime<chrono::Utc> {
    chrono::DateTime::parse_from_rfc3339(iso)
        .unwrap()
        .with_timezone(&chrono::Utc)
}

#[test]
fn data_persists_across_reopen() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("watch_log.db");

    // Insert through one store instance, then drop it.
    {
        let store = Store::open(&path).unwrap();
        let log = test_log(
            FilmId::Imdb("tt0137523".into()),
            "Fight Club",
            Venue::Theatre,
        )
        .with_notes("great movie");
        let stored = store.insert(log).unwrap();
        assert!(stored.id > 0);
    }

    // Reopen the same file: the row must still be there, intact.
    let store = Store::open(&path).unwrap();
    let fetched = store.get(1).unwrap().unwrap();
    assert_eq!(fetched.title, "Fight Club");
    assert_eq!(fetched.film_id, FilmId::Imdb("tt0137523".into()));
    assert_eq!(fetched.venue, Venue::Theatre);
    assert_eq!(fetched.notes.as_deref(), Some("great movie"));
    assert_eq!(fetched.format_summary, "4K laser · PLF");

    // A second log lists before the first (most recently watched first).
    let second = test_log(FilmId::Tmdb(551), "Second Film", Venue::Home);
    let stored_second = store.insert(second).unwrap();
    let listed = store.list().unwrap();
    assert_eq!(listed.len(), 2);
    assert_eq!(listed[0].id, stored_second.id);
}

#[test]
fn rewatch_and_watch_count_derived_from_prior_appearances() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("watch_log.db");
    let store = Store::open(&path).unwrap();

    // First viewing of the film: not a rewatch, watch count 1.
    let first = store
        .insert(test_log(
            FilmId::Imdb("tt0137523".into()),
            "Fight Club",
            Venue::Theatre,
        ))
        .unwrap();
    assert!(!first.rewatch);
    assert_eq!(first.watch_count, Some(1));

    // Second viewing of the same film: a rewatch, watch count 2.
    let second = store
        .insert(test_log(
            FilmId::Imdb("tt0137523".into()),
            "Fight Club",
            Venue::Theatre,
        ))
        .unwrap();
    assert!(second.rewatch);
    assert_eq!(second.watch_count, Some(2));

    // A different film is counted independently.
    let other = store
        .insert(test_log(
            FilmId::Imdb("tt0000001".into()),
            "Other",
            Venue::Home,
        ))
        .unwrap();
    assert!(!other.rewatch);
    assert_eq!(other.watch_count, Some(1));
}

#[test]
fn settings_default_and_selected_board_persists() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("watch_log.db");
    let store = Store::open(&path).unwrap();

    // Nothing stored yet: the default board.
    assert_eq!(store.settings().unwrap().selected_board, "US");

    store.set_selected_board("DE").unwrap();
    assert_eq!(store.settings().unwrap().selected_board, "DE");

    // Reopen: the selection persists.
    let store = Store::open(&path).unwrap();
    assert_eq!(store.settings().unwrap().selected_board, "DE");
}

#[test]
fn update_recomputes_chronological_rewatch_and_watch_count() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("watch_log.db");
    let store = Store::open(&path).unwrap();

    let mut first = test_log(
        FilmId::Imdb("tt0137523".into()),
        "Fight Club",
        Venue::Theatre,
    );
    first.watched_at = at("2020-01-01T12:00:00Z");
    let first = store.insert(first).unwrap();
    assert_eq!(first.watch_count, Some(1));

    // A later viewing of the same film.
    let mut second = test_log(
        FilmId::Imdb("tt0137523".into()),
        "Fight Club",
        Venue::Theatre,
    );
    second.watched_at = at("2021-01-01T12:00:00Z");
    let second = store.insert(second).unwrap();
    assert!(second.rewatch);
    assert_eq!(second.watch_count, Some(2));

    // `rewatch` means "not the first viewing in chronological order": the
    // earlier log stays the first viewing even after an edit.
    let mut updated = first.clone();
    updated.notes = Some("edited".into());
    let updated = store.update(updated).unwrap();
    assert!(!updated.rewatch);
    assert_eq!(updated.watch_count, Some(2));
    assert_eq!(updated.notes.as_deref(), Some("edited"));

    // Updating a log to a different film recounts against that film.
    let mut switched = updated;
    switched.film_id = FilmId::Imdb("tt0000001".into());
    let switched = store.update(switched).unwrap();
    assert!(!switched.rewatch);
    assert_eq!(switched.watch_count, Some(1));
}

#[test]
fn update_ignores_supplied_derived_fields() {
    let store = Store::open_in_memory().unwrap();
    let stored = store
        .insert(test_log(
            FilmId::Imdb("tt0137523".into()),
            "Fight Club",
            Venue::Theatre,
        ))
        .unwrap();
    let mut tampered = stored.clone();
    tampered.rewatch = true;
    tampered.watch_count = Some(99);
    let updated = store.update(tampered).unwrap();
    assert!(!updated.rewatch);
    assert_eq!(updated.watch_count, Some(1));
}

#[test]
fn delete_recounts_remaining() {
    let store = Store::open_in_memory().unwrap();
    let film_id = FilmId::Imdb("tt0137523".into());
    let mut first = test_log(film_id.clone(), "Fight Club", Venue::Theatre);
    first.watched_at = at("2020-01-01T12:00:00Z");
    let first = store.insert(first).unwrap();
    let mut second = test_log(film_id.clone(), "Fight Club", Venue::Theatre);
    second.watched_at = at("2021-01-01T12:00:00Z");
    let second = store.insert(second).unwrap();
    assert!(second.rewatch);

    // Deleting the later viewing makes the remaining one the only viewing.
    assert!(store.delete(second.id).unwrap());
    let remaining = store.get(first.id).unwrap().unwrap();
    assert_eq!(remaining.watch_count, Some(1));
    assert!(!remaining.rewatch);
}

#[test]
fn update_missing_returns_log_not_found() {
    let store = Store::open_in_memory().unwrap();
    let log = test_log(FilmId::Imdb("tt0137523".into()), "Fight Club", Venue::Home);
    let err = store.update(log).unwrap_err();
    assert!(matches!(err, Error::LogNotFound { id: 0 }));
}

#[test]
fn schema_newer_than_build_is_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("watch_log.db");

    // Stamp a schema version this build does not understand.
    {
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch("PRAGMA user_version = 3").unwrap();
    }

    let err = Store::open(&path).unwrap_err();
    assert!(matches!(err, Error::SchemaTooNew { version: 3 }));
}

#[test]
fn migration_v1_preserves_logs_and_builds_films() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("v1.db");

    // Build a v1 database by hand: one JSON document per row, no `films`.
    let mut earlier = test_log(
        FilmId::Imdb("tt0137523".into()),
        "Fight Club",
        Venue::Theatre,
    );
    earlier.watched_at = at("2020-01-01T12:00:00Z");
    earlier.release_year = Some(1999);
    let mut later = earlier.clone();
    later.watched_at = at("2021-01-01T12:00:00Z");
    {
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch(
            "CREATE TABLE watch_log (
                 id INTEGER PRIMARY KEY AUTOINCREMENT,
                 watched_at TEXT NOT NULL,
                 doc TEXT NOT NULL
             );
             CREATE TABLE settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);
             PRAGMA user_version = 1;",
        )
        .unwrap();
        for log in [&earlier, &later] {
            conn.execute(
                "INSERT INTO watch_log (watched_at, doc) VALUES (?1, ?2)",
                rusqlite::params![
                    log.watched_at.to_rfc3339(),
                    serde_json::to_string(log).unwrap()
                ],
            )
            .unwrap();
        }
    }

    let store = Store::open(&path).unwrap();
    let listed = store.list().unwrap();
    assert_eq!(listed.len(), 2);
    let films = store.list_films().unwrap();
    assert_eq!(films.len(), 1);
    assert_eq!(films[0].film.title, "Fight Club");
    assert_eq!(films[0].film.release_year, Some(1999));
    // Logs are ordered most recent first; only the later one is a rewatch.
    assert!(listed[0].rewatch);
    assert!(!listed[1].rewatch);
    assert_eq!(listed[0].watch_count, Some(2));

    let version: i64 = Connection::open(&path)
        .unwrap()
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .unwrap();
    assert_eq!(version, 2);
}

#[test]
fn import_rejects_invalid_file_and_keeps_current_data() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("watch_log.db");
    let junk = dir.path().join("junk.db");
    std::fs::write(&junk, b"not a database").unwrap();

    let mut store = Store::open(&path).unwrap();
    store
        .insert(test_log(
            FilmId::Imdb("tt0137523".into()),
            "Fight Club",
            Venue::Theatre,
        ))
        .unwrap();
    let err = store.import_from(&junk).unwrap_err();
    assert!(matches!(err, Error::InvalidImport { .. }));
    assert_eq!(store.list().unwrap().len(), 1);
}

#[test]
fn export_backup_opens_independently() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("watch_log.db");
    let exported = dir.path().join("exported.db");
    let store = Store::open(&path).unwrap();
    let stored = store
        .insert(test_log(
            FilmId::Imdb("tt0137523".into()),
            "Fight Club",
            Venue::Theatre,
        ))
        .unwrap();
    store.export_to(&exported).unwrap();

    let reopened = Store::open(&exported).unwrap();
    assert_eq!(reopened.list().unwrap(), vec![stored]);
    assert_eq!(reopened.list_films().unwrap().len(), 1);
}

#[test]
fn film_upsert_preserves_user_state() {
    let store = Store::open_in_memory().unwrap();
    let film_id = FilmId::Imdb("tt0137523".into());
    store
        .insert(test_log(film_id.clone(), "Fight Club", Venue::Theatre))
        .unwrap();
    store.set_favorite(&film_id, true).unwrap();
    store
        .set_tags(&film_id, &["noir".to_owned(), "classic".to_owned()])
        .unwrap();

    let mut film: FilmMetadata = store.get_film(&film_id).unwrap().unwrap();
    film.title = "Fight Club (Remastered)".into();
    film.metadata_refreshed_at = Some(chrono::Utc::now());
    store.upsert_film(&film).unwrap();

    let refreshed = store.get_film(&film_id).unwrap().unwrap();
    assert_eq!(refreshed.title, "Fight Club (Remastered)");
    assert!(refreshed.favorite);
    assert_eq!(
        refreshed.tags,
        vec!["noir".to_owned(), "classic".to_owned()]
    );
}
