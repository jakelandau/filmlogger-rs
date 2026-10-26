//! A live smoke test against the real TMDB API.
//!
//! Set `TMDB_API_KEY` to a valid TMDB API key, then run:
//!
//! ```sh
//! cargo run --example tmdb_smoke
//! ```

use film_logger::{Store, TmdbClient, Venue, WatchLog};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("TMDB_API_KEY").expect("set TMDB_API_KEY to a valid TMDB API key");

    let client = TmdbClient::new(api_key);

    // A full movie by its TMDB numeric id (Fight Club is 550).
    let movie = client.movie(550).await?;
    println!(
        "movie: {} ({}) runtime={}min vote={}",
        movie.title,
        movie.release_year().unwrap_or(0),
        movie.runtime.unwrap_or(0),
        movie.vote_average,
    );

    // The same film looked up by IMDB id.
    let found = client.find_by_imdb_id("tt0137523").await?;
    assert_eq!(found.id, 550);
    println!("find_by_imdb: {} -> id {}", found.title, found.id);

    // A search.
    let page = client.search_movies("fight club", 1).await?;
    println!(
        "search: {} result(s); first: {}",
        page.total_results,
        page.results.first().map_or("<none>", |r| r.title.as_str()),
    );

    // Record the viewing and persist it to a local SQLite file.
    let log = WatchLog::from_movie_now(&movie, Venue::Home);
    let store = Store::open("tmdb_smoke.db")?;
    let stored = store.insert(log)?;
    println!("stored log id {} for {}", stored.id, stored.title);

    Ok(())
}
