//! A live smoke test against the real TMDB API.
//!
//! Set `TMDB_API_KEY` in your build environment (it is baked in at compile
//! time), then run:
//!
//! ```sh
//! cargo run --example tmdb_smoke
//! ```

use film_logger::{Store, TmdbClient, Venue, WatchLog};
use film_logger::domain::{Format, HomeMedium};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = TmdbClient::from_compiled_key().expect(
        "set TMDB_API_KEY in the build environment before building (locally: \
         $env:TMDB_API_KEY=\"...\"; on CI: a GitHub Actions secret)",
    );

    // A full movie by its TMDB numeric id (Fight Club is 550).
    let movie = client.movie(550)?;
    println!(
        "movie: {} ({}) runtime={}min vote={}",
        movie.title,
        movie.release_year().unwrap_or(0),
        movie.runtime.unwrap_or(0),
        movie.vote_average,
    );

    // The same film looked up by IMDB id.
    let found = client.find_by_imdb_id("tt0137523")?;
    assert_eq!(found.id, 550);
    println!("find_by_imdb: {} -> id {}", found.title, found.id);

    // A search.
    let page = client.search_movies("fight club", 1)?;
    println!(
        "search: {} result(s); first: {}",
        page.total_results,
        page.results.first().map_or("<none>", |r| r.title.as_str()),
    );

    // Record the viewing and persist it to a local SQLite file.
    let log = WatchLog::from_movie_with_format_now(
        &movie,
        Venue::Home,
        Format::Home {
            medium: HomeMedium::BluRay,
        },
    )?;
    let store = Store::open("tmdb_smoke.db")?;
    let stored = store.insert(log)?;
    println!("stored log id {} for {}", stored.id, stored.title);

    Ok(())
}
