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
  installer builds for Windows, Linux, and macOS.
- Settings licenses/credits section and TMDB attribution.
- Film-strip icon for the Films navigation item.
- `CHANGELOG.md` for release-to-release change tracking.

### Changed

- `rewatch` and `watch_count` are now store-derived projections recomputed from
  each film's chronological viewing history.
- Database import validates the source before replacing live data.
- Android database, network, and TMDB calls run off the main thread.
- Android system back returns to the Logs screen before exiting the app.
- Rating-board icons now use bundled official symbols; the settings board still
  chooses which certification is shown in lists and film pages.

### Removed

- Film tags from the UI, Rust API, FFI surface, and database schema.

### Fixed

- Invalid database imports no longer replace the live database.
- Deleting a viewing correctly recounts the remaining viewings as rewatches or
  first viewings.
- Format summaries and legality checks no longer drift between desktop and
  Android because both consume the Rust catalog.
- Legacy logs with custom formats and legacy metadata remain readable.
