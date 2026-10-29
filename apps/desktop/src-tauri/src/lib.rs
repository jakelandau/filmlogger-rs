//! The desktop frontend: Tauri wiring over the [`film_logger`] facade.
//!
//! Tauri commands receive their arguments by value (deserialized from the
//! IPC message), so [`clippy::needless_pass_by_value`] is allowed crate-wide.
#![allow(clippy::needless_pass_by_value)]

use std::sync::Mutex;

use film_logger::{
    FilmLogger, Format, Movie, RatingBoard, SearchPage, Settings, Stats, TmdbClient, Venue,
    WatchLog,
};
use tauri::Manager;

/// The app's shared state: the logger, behind a mutex because commands may
/// need mutable access (e.g. import).
pub struct AppState {
    logger: Mutex<FilmLogger>,
}

/// The watch log, most recently watched first.
#[tauri::command]
fn list_logs(state: tauri::State<AppState>) -> Result<Vec<WatchLog>, String> {
    state
        .logger
        .lock()
        .unwrap()
        .list()
        .map_err(|e| e.to_string())
}

/// Log a viewing of a film found by TMDB id, enriching it from TMDB.
#[tauri::command]
fn add_film(
    state: tauri::State<AppState>,
    tmdb_id: i64,
    venue: Venue,
    format: Option<Format>,
) -> Result<WatchLog, String> {
    state
        .logger
        .lock()
        .unwrap()
        .add_film(tmdb_id, venue, format)
        .map_err(|e| e.to_string())
}

/// Update a stored log.
#[tauri::command]
fn update_log(state: tauri::State<AppState>, log: WatchLog) -> Result<WatchLog, String> {
    state
        .logger
        .lock()
        .unwrap()
        .update(log)
        .map_err(|e| e.to_string())
}

/// Delete a stored log. Returns whether a row was deleted.
#[tauri::command]
fn delete_log(state: tauri::State<AppState>, id: i64) -> Result<bool, String> {
    state
        .logger
        .lock()
        .unwrap()
        .delete(id)
        .map_err(|e| e.to_string())
}

/// Aggregate statistics over all logs.
#[tauri::command]
fn get_stats(state: tauri::State<AppState>) -> Result<Stats, String> {
    state
        .logger
        .lock()
        .unwrap()
        .stats()
        .map_err(|e| e.to_string())
}

/// The stored settings.
#[tauri::command]
fn get_settings(state: tauri::State<AppState>) -> Result<Settings, String> {
    state
        .logger
        .lock()
        .unwrap()
        .settings()
        .map_err(|e| e.to_string())
}

/// Persist the selected rating board.
#[tauri::command]
fn set_selected_board(state: tauri::State<AppState>, board: RatingBoard) -> Result<(), String> {
    state
        .logger
        .lock()
        .unwrap()
        .set_selected_board(board)
        .map_err(|e| e.to_string())
}

/// Export the database to `path`.
#[tauri::command]
fn export_to(state: tauri::State<AppState>, path: String) -> Result<(), String> {
    state
        .logger
        .lock()
        .unwrap()
        .export_to(path)
        .map_err(|e| e.to_string())
}

/// Replace the database contents from `path`.
#[tauri::command]
fn import_from(state: tauri::State<AppState>, path: String) -> Result<(), String> {
    let mut logger = state.logger.lock().unwrap();
    logger.import_from(path).map_err(|e| e.to_string())
}

/// Search TMDB for films.
#[tauri::command]
fn search_movies(
    state: tauri::State<AppState>,
    query: String,
    page: i32,
) -> Result<SearchPage, String> {
    state
        .logger
        .lock()
        .unwrap()
        .search(&query, page)
        .map_err(|e| e.to_string())
}

/// Find a film by IMDB id, resolving it to a full TMDB record.
#[tauri::command]
fn find_imdb(state: tauri::State<AppState>, imdb_id: String) -> Result<Movie, String> {
    state
        .logger
        .lock()
        .unwrap()
        .find_imdb(&imdb_id)
        .map_err(|e| e.to_string())
}

/// Start the desktop application.
///
/// # Panics
///
/// If the data directory cannot be created, no TMDB API key was compiled in,
/// the database cannot be opened, or the Tauri runtime fails to start.
pub fn run() {
    tauri::Builder::default()
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
                logger: Mutex::new(logger),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_logs,
            add_film,
            update_log,
            delete_log,
            get_stats,
            get_settings,
            set_selected_board,
            export_to,
            import_from,
            search_movies,
            find_imdb
        ])
        .run(tauri::generate_context!())
        .expect("while running the tauri application");
}
