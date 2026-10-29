//! Capture `TMDB_API_KEY` as a compile-time secret.
//!
//! Set `TMDB_API_KEY` in the build environment. Locally, the Android build
//! passes the key as the `-PtmdbApiKey` Gradle property (Gradle daemons
//! hide shell environment from cargo); other builds take it from the
//! environment directly. On GitHub Actions it is a repository secret. It is re-emitted as a compile-time
//! environment variable (`cargo:rustc-env`) and embedded in
//! [`film_logger::tmdb::COMPILED_API_KEY`] via `option_env!`, so a running
//! process never reads the environment. When the variable is absent the
//! crate still builds (the key is simply `None` at runtime).

fn main() {
    // Re-run this script (and rebuild the crate) whenever the key changes.
    println!("cargo:rerun-if-env-changed=TMDB_API_KEY");
    if let Ok(key) = std::env::var("TMDB_API_KEY") {
        println!("cargo:rustc-env=TMDB_API_KEY={key}");
    }
}
