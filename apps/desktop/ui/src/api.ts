// Thin wrappers over the Tauri IPC commands exposed by the `filmlogger-rs-desktop`
// desktop binary. Argument names are camelCase on the JS side; Tauri maps
// them to the Rust snake_case parameters. Struct fields keep their Rust
// snake_case names (Tauri only converts command argument names).
import { invoke } from "@tauri-apps/api/core";
import type {
  Collection,
  FilmListEntry,
  FilmMetadata,
  FilmPage,
  Format,
  FormatCatalog,
  Movie,
  NewLogDetails,
  RatingBoard,
  SearchPage,
  Settings,
  Stats,
  Venue,
  WatchLog,
} from "./types";

export const api = {
  listLogs: () => invoke<WatchLog[]>("list_logs"),
  addFilm: (tmdbId: number, venue: Venue, format: Format, details: NewLogDetails) =>
    invoke<WatchLog>("add_film", { tmdbId, venue, format, details }),
  logFilm: (
    filmId: string,
    venue: Venue,
    format: Format,
    details: NewLogDetails
  ) => invoke<WatchLog>("log_film", { filmId, venue, format, details }),
  updateLog: (log: WatchLog) => invoke<WatchLog>("update_log", { log }),
  deleteLog: (id: number) => invoke<boolean>("delete_log", { id }),
  getStats: () => invoke<Stats>("get_stats"),
  getSettings: () => invoke<Settings>("get_settings"),
  ratingBoards: () => invoke<RatingBoard[]>("rating_boards"),
  setSelectedBoard: (board: string) =>
    invoke<void>("set_selected_board", { board }),
  exportTo: (path: string) => invoke<void>("export_to", { path }),
  importFrom: (path: string) => invoke<void>("import_from", { path }),
  searchMovies: (query: string, page: number) =>
    invoke<SearchPage>("search_movies", { query, page }),
  popularMovies: (page: number) => invoke<SearchPage>("popular_movies", { page }),
  findImdb: (imdbId: string) => invoke<Movie>("find_imdb", { imdbId }),
  refreshFilm: (filmId: string) =>
    invoke<FilmMetadata>("refresh_film", { filmId }),
  listFilms: () => invoke<FilmListEntry[]>("list_films"),
  getFilmPage: (filmId: string) => invoke<FilmPage>("get_film_page", { filmId }),
  getCollection: (collectionId: number) =>
    invoke<Collection>("get_collection", { collectionId }),
  ensureFilm: (tmdbId: number) => invoke<FilmMetadata>("ensure_film", { tmdbId }),
  addToWatchlist: (tmdbId: number) =>
    invoke<FilmMetadata>("add_to_watchlist", { tmdbId }),
  setFavorite: (filmId: string, favorite: boolean) =>
    invoke<FilmMetadata>("set_favorite", { filmId, favorite }),
  setWatchlisted: (filmId: string, watchlisted: boolean) =>
    invoke<FilmMetadata>("set_watchlisted", { filmId, watchlisted }),
  formatCatalog: () => invoke<FormatCatalog>("format_catalog"),
};
