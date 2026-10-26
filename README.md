# FilmLoggerRust

A monorepo for the **FilmLogger** products — a personal film watch-log backed by
the [TMDB API](https://developer.themoviedb.org/docs/getting-started) and
persisted in a single-file SQLite database shared by all products.

Three products live here:

| # | Product | Location | Status |
|---|---------|----------|--------|
| 1 | Core library (Rust) | `crates/film-logger` | **this work** |
| 2 | Desktop app (Windows / Linux) | `apps/desktop` (planned) | — |
| 3 | Android native app | `apps/android` (planned) | — |

The core library (`film-logger`) records each film you watch — **when**
(device-local wall time translated to UTC via the device's current timezone),
**where** (theatre or home), and other details (notes, denormalized film
metadata). Films are identified by **IMDB ID** (e.g. `tt0137523`), falling back
to the TMDB numeric ID when a film has no IMDB ID. It is async (tokio/reqwest)
and platform-agnostic so the desktop and Android apps can consume it directly.

## Quickstart (product 1: `crates/film-logger`)

```sh
# Build
cargo build

# Run tests (domain unit + wiremock HTTP + SQLite round-trip)
cargo test --workspace

# Live smoke test against api.themoviedb.org (needs a TMDB key)
cargo run -p film-logger --example tmdb_smoke
```

The live smoke test requires a TMDB **API Read Access Token**, exported as the
`TMDB_API_KEY` environment variable. Obtain one at
<https://www.themoviedb.org/settings/api>:

```sh
export TMDB_API_KEY="<your token>"   # Windows (PowerShell): $env:TMDB_API_KEY="..."
```

## License

Licensed under the [GNU Lesser General Public License v3 (or later)](LICENSE).
