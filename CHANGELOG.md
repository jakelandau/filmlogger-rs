# Changelog

All notable changes to FilmLogger are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project uses
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Until the first release tag is created, changes stay under **Unreleased**. When
tagging, rename that heading to the release version and add the date, for
example `## [0.1.0] - 2026-11-15`.

## [Unreleased]

### Added

- SQLite schema v3 with a `films` table for metadata, favorites, and user state,
  plus migrations from the original single-table watch log.
- Film library and per-film pages on desktop and Android, including viewing
  history, favorite toggles, metadata refresh, and logging another viewing.
- Add-flow details on both platforms: watched-at date/time, rating, and notes.
- Shared Rust format catalog driving the desktop and Android format pickers,
  with format legality and presentation-flag rules defined once.
- Rich TMDB metadata: cast, collection, overview, tagline, runtime, genres,
  and board certifications.
- Official rating symbols bundled from Wikimedia Commons, with text fallback
  for boards or ratings without an official symbol.
- Log filters and sorting on both platforms, plus paginated search and popular
  film results.
- Expanded statistics breakdowns by month, projection family, screen, medium,
  and presentation flag.
- Native desktop database import/export dialogs and backup-based export/import.
- CI for the Rust core, desktop UI, and Android APK, plus tagged-release
  installer builds for Windows and Linux and a signed Android release APK.
- Automatic update checks on launch, plus manual “Check for Updates” buttons
  on desktop and Android. Releases publish Tauri updater signatures and
  `latest.json`; Android downloads the release APK and opens the installer.
- Signing and release documentation, including the required Actions secrets.
- Third-party license inventory in `THIRD_PARTY.md`.
- Home-format presentation flags: 3D Blu-ray is labelled and forced to 3D;
  4K Ultra HD Blu-ray can optionally record HFR.
- Settings licenses/credits section and TMDB attribution.
- Film-strip icon for the Films navigation item.
- `CHANGELOG.md` for release-to-release change tracking.
- Stats now uses every field a viewing carries: rating counts, genres, total
  runtime, and first/last viewing dates, plus weekday and hour-of-day
  breakdowns resolved in each log's own time zone.
- Ratings-by-thumbs sorting on the Logs list (up, then down, then unrated,
  with recency as the tie-break).
- Flag and name for every rating board in Settings, and an export/import
  status line.

### Changed

- The project is now licensed under **AGPL-3.0-or-later** instead of
  LGPL-3.0-or-later, so hosting a modified copy as a network service requires
  offering its source to that service's users. Every shipped dependency is
  AGPL-compatible; see the compatibility table in `THIRD_PARTY.md`.
- `rewatch` and `watch_count` are now store-derived projections recomputed from
  each film's chronological viewing history.
- Database import validates the source before replacing live data.
- Android database, network, and TMDB calls run off the main thread.
- Android system back returns to the Logs screen before exiting the app.
- Rating-board icons now use bundled official symbols; the settings board still
  chooses which certification is shown in lists and film pages.
- Desktop and Android now show the same content, actions, strings, and computed
  values on every screen; only native chrome, widget type, and layout differ.
- The film page renders only the selected board's certification, at a larger
  size for legibility, instead of every stored board.
- Viewing times on Android include the short time-zone name, matching desktop.
- Android's document picker accepts any file type for import, so a desktop
  export synced through Drive or Downloads can be chosen.
- Importing a database invalidates each app's cached film and collection data.
- Desktop database and TMDB commands now all run on Tauri's blocking pool, so a
  list reload, export, or import no longer freezes the window.
- Reading one viewing or one film page no longer loads the whole library, and
  the Stats tab aggregates without hydrating logs, so large databases open
  faster.
- The film-library payload carries only the fields the Films tab and collection
  cards render; full metadata loads with a film page.
- Android now takes its format keys and film keys from the Rust core instead of
  its own tables, so a core key change cannot leave the app behind.

### Removed

- Film tags from the UI, Rust API, FFI surface, and database schema.
- The duplicate in-content back link on Android's film screen; the app bar's
  back arrow is the single back affordance.

### Fixed

- Invalid database imports no longer replace the live database.
- Exporting over the live database path is rejected instead of deleting it.
- Deleting a viewing correctly recounts the remaining viewings as rewatches or
  first viewings.
- Format summaries and legality checks no longer drift between desktop and
  Android because both consume the Rust catalog.
- Legacy logs with custom formats and legacy metadata remain readable.
