// Thin wrappers over the Tauri IPC commands exposed by the `film-logger`
// desktop binary. Argument names are camelCase on the JS side; Tauri maps
// them to the Rust snake_case parameters.
import type {
  Format,
  Movie,
  RatingBoard,
  SearchPage,
  Settings,
  Stats,
  Venue,
  WatchLog,
} from "./types";

declare global {
  interface Window {
    __TAURI__?: {
      core: {
        invoke: <T>(cmd: string, args?: Record<string, unknown>) => Promise<T>;
      };
    };
  }
}

function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const core = window.__TAURI__?.core;
  if (!core) {
    return Promise.reject(new Error("Tauri runtime not available"));
  }
  return core.invoke<T>(cmd, args);
}

export const api = {
  listLogs: () => invoke<WatchLog[]>("list_logs"),
  addFilm: (tmdbId: number, venue: Venue, format: Format | null) =>
    invoke<WatchLog>("add_film", { tmdbId: tmdbId, venue, format }),
  updateLog: (log: WatchLog) => invoke<WatchLog>("update_log", { log }),
  deleteLog: (id: number) => invoke<boolean>("delete_log", { id }),
  getStats: () => invoke<Stats>("get_stats"),
  getSettings: () => invoke<Settings>("get_settings"),
  setSelectedBoard: (board: RatingBoard) =>
    invoke<void>("set_selected_board", { board }),
  exportTo: (path: string) => invoke<void>("export_to", { path }),
  importFrom: (path: string) => invoke<void>("import_from", { path }),
  searchMovies: (query: string, page: number) =>
    invoke<SearchPage>("search_movies", { query, page }),
  findImdb: (imdbId: string) => invoke<Movie>("find_imdb", { imdbId: imdbId }),
};
