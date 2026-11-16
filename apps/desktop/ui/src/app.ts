// Application shell: shared state, navigation, and the render loop. The
// whole content region is re-rendered on every state change; views are
// pure functions of state (plus their own imperative sub-widgets).
import { el, clear } from "./dom";
import { api } from "./api";
import { initCatalog } from "./format-ui";
import { icon } from "./icons";
import { checkForUpdates } from "./updates";
import type {
  FilmListEntry,
  FilmPage,
  FormatCatalog,
  RatingBoard,
  Settings,
  Stats,
  WatchLog,
} from "./types";
import { renderLogs } from "./views/logs";
import { renderAdd } from "./views/add";
import { renderEdit } from "./views/edit";
import { renderStats } from "./views/stats";
import { renderSettings } from "./views/settings";
import { renderFilms } from "./views/films";
import { renderFilm } from "./views/film";

export type View = "logs" | "films" | "film" | "add" | "edit" | "stats" | "settings";

interface State {
  view: View;
  editId: number | null;
  filmId: string | null;
  logs: WatchLog[];
  films: FilmListEntry[];
  settings: Settings;
  ratingBoards: RatingBoard[];
  stats: Stats | null;
  catalog: FormatCatalog | null;
  filmPage: FilmPage | null;
}

const state: State = {
  view: "logs",
  editId: null,
  filmId: null,
  logs: [],
  films: [],
  settings: { selected_board: "US" },
  ratingBoards: [],
  stats: null,
  catalog: null,
  filmPage: null,
};

export function getState(): State {
  return state;
}

// Refresh `logs`, `films`, `settings`, `ratingBoards`, and `stats` from the store.
export async function reload(): Promise<void> {
  const [logs, films, settings, boards, stats] = await Promise.all([
    api.listLogs(),
    api.listFilms(),
    api.getSettings(),
    api.ratingBoards(),
    api.getStats(),
  ]);
  state.logs = logs;
  state.films = films;
  state.settings = settings;
  state.ratingBoards = boards;
  state.stats = stats;
}

// Re-render the current view (e.g. after a mutation that changed state).
export function rerender(): void {
  render();
}

// Change the active view. A number target is an edit id; a string target is
// a film id.
export function go(view: View, target?: number | string): void {
  state.view = view;
  state.editId = typeof target === "number" ? target : null;
  state.filmId = typeof target === "string" ? target : null;
  if (view !== "film") state.filmPage = null;
  render();
  if (view === "film" && state.filmId !== null) {
    void loadFilmPage(state.filmId);
  }
}

async function loadFilmPage(filmId: string): Promise<void> {
  try {
    const page = await api.getFilmPage(filmId);
    if (state.view === "film" && state.filmId === filmId) {
      state.filmPage = page;
      render();
    }
  } catch (e) {
    showError(e);
  }
}

export async function init(): Promise<void> {
  let failure: unknown;
  try {
    const catalog = await api.formatCatalog();
    state.catalog = catalog;
    initCatalog(catalog);
    await reload();
  } catch (e) {
    failure = e;
  }
  render();
  if (failure !== undefined) {
    showError(failure);
  }
  void checkForUpdates(true);
}

function render(): void {
  const root = document.getElementById("root");
  if (!root) return;
  clear(root);
  root.append(topBar());
  const content = el("main", { class: "content" });
  renderView(content);
  root.append(content);
  if (state.view === "logs") {
    root.append(
      el(
        "button",
        {
          class: "fab",
          title: "Add a film",
          "aria-label": "Add a film",
          onclick: () => go("add"),
        },
        icon("plus")
      )
    );
  }
}

function renderView(content: HTMLElement): void {
  switch (state.view) {
    case "logs":
      return renderLogs(content);
    case "films":
      return renderFilms(content);
    case "film":
      return renderFilm(content);
    case "add":
      return renderAdd(content);
    case "edit":
      return renderEdit(content);
    case "stats":
      return renderStats(content);
    case "settings":
      return renderSettings(content);
  }
}

function topBar(): HTMLElement {
  if (
    state.view === "logs" ||
    state.view === "films" ||
    state.view === "stats" ||
    state.view === "settings"
  ) {
    return nav();
  }
  const title =
    state.view === "add" ? "Add film" : state.view === "edit" ? "Edit viewing" : "Film";
  const back = state.view === "film" ? "films" : "logs";
  return el(
    "header",
    { class: "top-app-bar" },
    el(
      "button",
      {
        class: "icon-button",
        title: "Back",
        "aria-label": "Back",
        onclick: () => go(back),
      },
      icon("back")
    ),
    el("h1", { class: "top-title" }, title),
    el("span")
  );
}

function nav(): HTMLElement {
  const tabs: [View, string, "logs" | "films" | "stats" | "settings"][] = [
    ["logs", "Logs", "logs"],
    ["films", "Films", "films"],
    ["stats", "Stats", "stats"],
    ["settings", "Settings", "settings"],
  ];
  return el(
    "nav",
    { class: "nav" },
    el("span", { class: "brand" }, "FilmLogger"),
    ...tabs.map(([view, label, iconName]) =>
      el(
        "button",
        {
          class: state.view === view || (view === "films" && state.view === "film") ? "tab active" : "tab",
          onclick: () => go(view),
        },
        el("span", { class: "tab-icon" }, icon(iconName)),
        el("span", { class: "tab-label" }, label)
      )
    )
  );
}

// Surface a rejected IPC promise as a transient banner at the top of the
// content area.
export function showError(e: unknown): void {
  const message = e instanceof Error ? e.message : String(e);
  const banner = el("div", { class: "error-banner" }, "⚠ ", message);
  const root = document.getElementById("root");
  if (!root) return;
  root.prepend(banner);
  setTimeout(() => banner.remove(), 6000);
}
