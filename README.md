# FilmLoggerRust

A monorepo for the **FilmLogger** products — a personal film watch-log backed by
the [TMDB API](https://developer.themoviedb.org/docs/getting-started) and
persisted in a single-file SQLite database shared by all products.

Three products live here:

| # | Product | Location | Status |
|---|---------|----------|--------|
| 1 | Core library (Rust) | `crates/filmlogger-rs` | Stable |
| 2 | Desktop app (Windows / Linux) | `apps/desktop` | Stable |
| 3 | Android native app | `apps/android` | Stable |

The core library (`filmlogger-rs`) records each film you watch — **when**
(device-local wall time translated to UTC via the device's current timezone),
**where** (theatre or home), and other details (notes, denormalized film
metadata). Films are identified by **IMDB ID** (e.g. `tt0137523`), falling back
to the TMDB numeric ID when a film has no IMDB ID. Its TMDB client is a
blocking `reqwest` client, so every app calls it from a worker thread; it is
platform-agnostic, and the desktop and Android apps consume it directly (the
Android app through a `uniffi` bridge).

## Quickstart (product 1: `crates/filmlogger-rs`)

```sh
# Build
cargo build

# Run tests (domain unit + wiremock HTTP + SQLite round-trip)
cargo test --workspace

# Live smoke test against api.themoviedb.org (needs a TMDB key)
cargo run -p filmlogger-rs --example tmdb_smoke
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

Licensed under the [GNU Affero General Public License v3 (or later)](LICENSE).

FilmLogger is a local desktop and Android app, but the AGPL closes the
application-service loophole: anyone who hosts a modified version of this code
as a network service must offer its complete corresponding source to that
service's users. Third-party dependency licenses remain compatible with
AGPL-3.0-or-later; see [`THIRD_PARTY.md`](THIRD_PARTY.md) for the inventory and
the compatibility notes.
