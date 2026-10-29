// Application shell: shared state, navigation, and the render loop. The
// whole content region is re-rendered on every state change; views are
// pure functions of state (plus their own imperative sub-widgets).
import { el, clear } from "./dom";
import { api } from "./api";
import type { Settings, Stats, WatchLog } from "./types";
import { renderLogs } from "./views/logs";
import { renderAdd } from "./views/add";
import { renderEdit } from "./views/edit";
import { renderStats } from "./views/stats";
import { renderSettings } from "./views/settings";

export type View = "logs" | "add" | "edit" | "stats" | "settings";

interface State {
  view: View;
  editId: number | null;
  logs: WatchLog[];
  settings: Settings;
  stats: Stats | null;
}

const state: State = {
  view: "logs",
  editId: null,
  logs: [],
  settings: { selected_board: "mpa" },
  stats: null,
};

export function getState(): State {
  return state;
}

// Refresh `logs`, `settings`, and `stats` from the store.
export async function reload(): Promise<void> {
  const [logs, settings, stats] = await Promise.all([
    api.listLogs(),
    api.getSettings(),
    api.getStats(),
  ]);
  state.logs = logs;
  state.settings = settings;
  state.stats = stats;
}

// Re-render the current view (e.g. after a mutation that changed state).
export function rerender(): void {
  render();
}

// Change the active view. `editId` is only meaningful for the `edit` view.
export function go(view: View, editId?: number): void {
  state.view = view;
  state.editId = editId ?? null;
  render();
}

export async function init(): Promise<void> {
  let failure: unknown;
  try {
    await reload();
  } catch (e) {
    failure = e;
  }
  render();
  if (failure !== undefined) {
    showError(failure);
  }
}

function render(): void {
  const root = document.getElementById("root");
  if (!root) return;
  clear(root);
  root.append(nav());
  const content = el("main", { class: "content" });
  renderView(content);
  root.append(content);
}

function renderView(content: HTMLElement): void {
  switch (state.view) {
    case "logs":
      return renderLogs(content);
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

function nav(): HTMLElement {
  const tabs: [View, string][] = [
    ["logs", "Logs"],
    ["add", "Add"],
    ["stats", "Stats"],
    ["settings", "Settings"],
  ];
  return el(
    "nav",
    { class: "nav" },
    el("span", { class: "brand" }, "FilmLogger"),
    ...tabs.map(([view, label]) =>
      el(
        "button",
        {
          class: state.view === view ? "tab active" : "tab",
          onclick: () => go(view),
        },
        label
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
