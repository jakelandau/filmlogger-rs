//! The desktop frontend: Tauri wiring over the [`film_logger`] facade.
//!
//! Blocking work (TMDB requests, SQLite reads/writes) runs on Tauri's
//! blocking pool whenever it can, so the IPC thread never blocks the webview.
//!
//! Tauri commands receive their arguments by value (deserialized from the
//! IPC message), so [`clippy::needless_pass_by_value`] is allowed crate-wide.
#![allow(clippy::needless_pass_by_value)]

use std::sync::{Arc, Mutex};

use film_logger::{
    Collection, FilmId, FilmListEntry, FilmLogger, FilmMetadata, FilmPage, Format, FormatCatalog,
    Movie, NewLogDetails, RatingBoard, SearchPage, Settings, Stats, TmdbClient, Venue, WatchLog,
};
use tauri::Manager;

/// The app's shared state: the logger, behind a mutex because commands may
/// need mutable access (e.g. import).
pub struct AppState {
    logger: Arc<Mutex<FilmLogger>>,
}

/// Run a blocking logger operation on the blocking pool.
async fn run_blocking<T, F>(state: &AppState, f: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce(&FilmLogger) -> Result<T, film_logger::Error> + Send + 'static,
{
    let logger = Arc::clone(&state.logger);
    tauri::async_runtime::spawn_blocking(move || {
        let logger = logger.lock().map_err(|e| e.to_string())?;
        f(&logger).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Parse an IPC film id key (`tt0137523` / `tmdb:550`).
fn parse_film_id(key: &str) -> Result<FilmId, String> {
    FilmId::from_key(key).map_err(|e| e.to_string())
}

/// The watch log, most recently watched first.
#[tauri::command]
fn list_logs(state: tauri::State<'_, AppState>) -> Result<Vec<WatchLog>, String> {
    state
        .logger
        .lock()
        .map_err(|e| e.to_string())?
        .list()
        .map_err(|e| e.to_string())
}

/// Log a viewing of a film found by TMDB id, enriching it from TMDB.
#[tauri::command]
async fn add_film(
    state: tauri::State<'_, AppState>,
    tmdb_id: i64,
    venue: Venue,
    format: Format,
    details: NewLogDetails,
) -> Result<WatchLog, String> {
    run_blocking(&state, move |logger| {
        logger.add_film(tmdb_id, venue, format, &details)
    })
    .await
}

/// Log a viewing of a stored film (no network).
#[tauri::command]
async fn log_film(
    state: tauri::State<'_, AppState>,
    film_id: String,
    venue: Venue,
    format: Format,
    details: NewLogDetails,
) -> Result<WatchLog, String> {
    let key = film_id;
    run_blocking(&state, move |logger| {
        let film_id = FilmId::from_key(&key)?;
        logger.log_film(&film_id, venue, format, &details)
    })
    .await
}

/// Update a stored log.
#[tauri::command]
fn update_log(state: tauri::State<'_, AppState>, log: WatchLog) -> Result<WatchLog, String> {
    state
        .logger
        .lock()
        .map_err(|e| e.to_string())?
        .update(log)
        .map_err(|e| e.to_string())
}

/// Delete a stored log. Returns whether a row was deleted.
#[tauri::command]
fn delete_log(state: tauri::State<'_, AppState>, id: i64) -> Result<bool, String> {
    state
        .logger
        .lock()
        .map_err(|e| e.to_string())?
        .delete(id)
        .map_err(|e| e.to_string())
}

/// Aggregate statistics over all logs.
#[tauri::command]
fn get_stats(state: tauri::State<'_, AppState>) -> Result<Stats, String> {
    state
        .logger
        .lock()
        .map_err(|e| e.to_string())?
        .stats()
        .map_err(|e| e.to_string())
}

/// The stored settings.
#[tauri::command]
fn get_settings(state: tauri::State<'_, AppState>) -> Result<Settings, String> {
    state
        .logger
        .lock()
        .map_err(|e| e.to_string())?
        .settings()
        .map_err(|e| e.to_string())
}

/// Every rating board TMDB reports certifications for.
#[tauri::command]
fn rating_boards(state: tauri::State<'_, AppState>) -> Result<Vec<RatingBoard>, String> {
    Ok(state
        .logger
        .lock()
        .map_err(|e| e.to_string())?
        .rating_boards())
}

/// Persist the selected rating board by region code.
#[tauri::command]
fn set_selected_board(state: tauri::State<'_, AppState>, board: String) -> Result<(), String> {
    state
        .logger
        .lock()
        .map_err(|e| e.to_string())?
        .set_selected_board(&board)
        .map_err(|e| e.to_string())
}

/// Export the database to `path`.
#[tauri::command]
fn export_to(state: tauri::State<'_, AppState>, path: String) -> Result<(), String> {
    state
        .logger
        .lock()
        .map_err(|e| e.to_string())?
        .export_to(path)
        .map_err(|e| e.to_string())
}

/// Replace the database contents from `path`.
#[tauri::command]
fn import_from(state: tauri::State<'_, AppState>, path: String) -> Result<(), String> {
    let mut logger = state.logger.lock().map_err(|e| e.to_string())?;
    logger.import_from(path).map_err(|e| e.to_string())
}

/// Search TMDB for films.
#[tauri::command]
async fn search_movies(
    state: tauri::State<'_, AppState>,
    query: String,
    page: i32,
) -> Result<SearchPage, String> {
    run_blocking(&state, move |logger| logger.search(&query, page)).await
}

/// The currently popular movies.
#[tauri::command]
async fn popular_movies(
    state: tauri::State<'_, AppState>,
    page: i32,
) -> Result<SearchPage, String> {
    run_blocking(&state, move |logger| logger.popular_movies(page)).await
}

/// Find a film by IMDB id, resolving it to a full TMDB record.
#[tauri::command]
async fn find_imdb(state: tauri::State<'_, AppState>, imdb_id: String) -> Result<Movie, String> {
    run_blocking(&state, move |logger| logger.find_imdb(&imdb_id)).await
}

/// Re-fetch a stored film's metadata from TMDB.
#[tauri::command]
async fn refresh_film(
    state: tauri::State<'_, AppState>,
    film_id: String,
) -> Result<FilmMetadata, String> {
    let key = film_id;
    run_blocking(&state, move |logger| {
        logger.refresh_film(&FilmId::from_key(&key)?)
    })
    .await
}

/// The film library, with viewing summaries.
#[tauri::command]
fn list_films(state: tauri::State<'_, AppState>) -> Result<Vec<FilmListEntry>, String> {
    state
        .logger
        .lock()
        .map_err(|e| e.to_string())?
        .films()
        .map_err(|e| e.to_string())
}

/// One film's page: metadata, history, and statistics.
#[tauri::command]
fn get_film_page(state: tauri::State<'_, AppState>, film_id: String) -> Result<FilmPage, String> {
    let key = film_id;
    state
        .logger
        .lock()
        .map_err(|e| e.to_string())?
        .film_page(&parse_film_id(&key)?)
        .map_err(|e| e.to_string())
}

/// Mark a film a favorite (or clear it).
#[tauri::command]
fn set_favorite(
    state: tauri::State<'_, AppState>,
    film_id: String,
    favorite: bool,
) -> Result<FilmMetadata, String> {
    state
        .logger
        .lock()
        .map_err(|e| e.to_string())?
        .set_favorite(&parse_film_id(&film_id)?, favorite)
        .map_err(|e| e.to_string())
}

/// Fetch a TMDB collection and its films.
#[tauri::command]
async fn get_collection(
    state: tauri::State<'_, AppState>,
    collection_id: i64,
) -> Result<Collection, String> {
    run_blocking(&state, move |logger| logger.collection(collection_id)).await
}

/// Fetch and store a TMDB film without logging a viewing.
#[tauri::command]
async fn ensure_film(
    state: tauri::State<'_, AppState>,
    tmdb_id: i64,
) -> Result<FilmMetadata, String> {
    run_blocking(&state, move |logger| logger.ensure_film(tmdb_id)).await
}

/// Add a TMDB film to the bucket list, storing it if needed.
#[tauri::command]
async fn add_to_watchlist(
    state: tauri::State<'_, AppState>,
    tmdb_id: i64,
) -> Result<FilmMetadata, String> {
    run_blocking(&state, move |logger| logger.add_to_watchlist(tmdb_id)).await
}

/// The viewing-format catalog.
#[tauri::command]
fn format_catalog() -> FormatCatalog {
    film_logger::format_catalog()
}

/// Start the desktop application.
///
/// # Panics
///
/// If the data directory cannot be created, no TMDB API key was compiled in,
/// the database cannot be opened, or the Tauri runtime fails to start.
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        // The app is always dark, including the window chrome.
        .on_page_load(|webview, _| {
            let _ = webview.window().set_theme(Some(tauri::Theme::Dark));
        })
        .setup(|app| {
            let dir = app.path().app_config_dir()?;
            std::fs::create_dir_all(&dir)?;
            let client = TmdbClient::from_compiled_key()
                .ok_or_else(|| "no TMDB API key was compiled in".to_string())?;
            let logger = FilmLogger::open_with(dir.join("watch_log.db"), client)?;
            app.manage(AppState {
                logger: Arc::new(Mutex::new(logger)),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_logs,
            add_film,
            log_film,
            update_log,
            delete_log,
            get_stats,
            get_settings,
            rating_boards,
            set_selected_board,
            export_to,
            import_from,
            search_movies,
            popular_movies,
            find_imdb,
            refresh_film,
            list_films,
            get_film_page,
            set_favorite,
            get_collection,
            ensure_film,
            add_to_watchlist,
            format_catalog
        ])
        .run(tauri::generate_context!())
        .expect("while running the tauri application");
}
