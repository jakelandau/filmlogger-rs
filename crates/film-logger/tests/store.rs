//! Integration tests for the SQLite store, using a temp file to verify
//! persistence across store instances.

use film_logger::{FilmId, Store, Venue, WatchLog};

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
