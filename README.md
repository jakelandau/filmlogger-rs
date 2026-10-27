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

The live smoke test needs the TMDB key **baked in at compile time**. Set
`TMDB_API_KEY` in your build environment — it is captured by `build.rs` and
embedded via `option_env!` (exposed as `TmdbClient::from_compiled_key()` /
`COMPILED_API_KEY`) — then build and run. Either key type works: an **API Read
Access Token** (a JWT, starting with `eyJ`) is sent as a `Bearer` token, and a
legacy **API Key** is sent as `?api_key=`. Obtain the key at
<https://www.themoviedb.org/settings/api>.

- **Locally** — PowerShell: `$env:TMDB_API_KEY = "..."` · bash/zsh:
  `export TMDB_API_KEY="..."` (set before `cargo build` / `cargo run`).
- **GitHub Actions / CI-CD** — store it as a repository **secret** named
  `TMDB_API_KEY`; `.github/workflows/ci.yml` injects it into the build.

`cargo test` needs no key (it uses a local wiremock server); only the live
smoke does. The key never lands in git (it lives in your shell locally and in
a repository secret on CI); `.env` is gitignored.

## License

Licensed under the [GNU Lesser General Public License v3 (or later)](LICENSE).
