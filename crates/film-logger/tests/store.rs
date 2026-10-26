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
