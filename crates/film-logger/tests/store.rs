//! Integration tests for the SQLite store, using a temp file to verify
//! persistence across store instances.

use film_logger::{Error, FilmId, RatingBoard, Store, Venue, WatchLog};

#[test]
fn data_persists_across_reopen() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("watch_log.db");

    // Insert through one store instance, then drop it.
    {
        let store = Store::open(&path).unwrap();
        let log = WatchLog::now(
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

    // A second log lists before the first (most recently watched first).
    let second = WatchLog::now(FilmId::Tmdb(551), "Second Film", Venue::Home);
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
        .insert(WatchLog::now(
            FilmId::Imdb("tt0137523".into()),
            "Fight Club",
            Venue::Theatre,
        ))
        .unwrap();
    assert!(!first.rewatch);
    assert_eq!(first.watch_count, Some(1));

    // Second viewing of the same film: a rewatch, watch count 2.
    let second = store
        .insert(WatchLog::now(
            FilmId::Imdb("tt0137523".into()),
            "Fight Club",
            Venue::Theatre,
        ))
        .unwrap();
    assert!(second.rewatch);
    assert_eq!(second.watch_count, Some(2));

    // A different film is counted independently.
    let other = store
        .insert(WatchLog::now(
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
    assert_eq!(store.settings().unwrap().selected_board, RatingBoard::Mpa);

    store.set_selected_board(RatingBoard::Fsk).unwrap();
    assert_eq!(store.settings().unwrap().selected_board, RatingBoard::Fsk);

    // Reopen: the selection persists.
    let store = Store::open(&path).unwrap();
    assert_eq!(store.settings().unwrap().selected_board, RatingBoard::Fsk);
}

#[test]
fn update_recomputes_rewatch_and_watch_count() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("watch_log.db");
    let store = Store::open(&path).unwrap();

    let first = store
        .insert(WatchLog::now(
            FilmId::Imdb("tt0137523".into()),
            "Fight Club",
            Venue::Theatre,
        ))
        .unwrap();
    assert_eq!(first.watch_count, Some(1));

    // A second viewing of the same film.
    let second = store
        .insert(WatchLog::now(
            FilmId::Imdb("tt0137523".into()),
            "Fight Club",
            Venue::Theatre,
        ))
        .unwrap();
    assert!(second.rewatch);
    assert_eq!(second.watch_count, Some(2));

    // Updating the first log recounts it: the film now has another viewing
    // on record, so the update flags it as a rewatch with the running count.
    let mut updated = first.clone();
    updated.notes = Some("edited".into());
    let updated = store.update(updated).unwrap();
    assert!(updated.rewatch);
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
fn update_missing_returns_log_not_found() {
    let store = Store::open_in_memory().unwrap();
    let log = WatchLog::now(FilmId::Imdb("tt0137523".into()), "Fight Club", Venue::Home);
    let err = store.update(log).unwrap_err();
    assert!(matches!(err, Error::LogNotFound { id: 0 }));
}

#[test]
fn schema_newer_than_build_is_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("watch_log.db");

    // Stamp a schema version this build does not understand.
    {
        let conn = rusqlite::Connection::open(&path).unwrap();
        conn.execute_batch("PRAGMA user_version = 2").unwrap();
    }

    let err = Store::open(&path).unwrap_err();
    assert!(matches!(err, Error::SchemaTooNew { version: 2 }));
}
