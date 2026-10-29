// Edit a stored log: title, venue, format, watched-at, notes, rating,
// rewatch, watch count. Editable fields are captured into module state
// before any re-render so imperative widgets survive the full re-render.
import { el } from "../dom";
import { api } from "../api";
import { getState, go, reload, showError } from "../app";
import { defaultFormat, formatControls } from "../format-ui";
import type { FormatControls } from "../format-ui";
import { fromLocalInput, toLocalInput } from "../time";
import type { Format, Rating, Venue, WatchLog } from "../types";

interface EditState {
  id: number | null;
  title: string;
  watchedLocal: string;
  notes: string;
  rating: "none" | Rating;
  rewatch: boolean;
  watchCountText: string;
  venue: Venue;
  format: Format | null;
}

const edit: EditState = {
  id: null,
  title: "",
  watchedLocal: "",
  notes: "",
  rating: "none",
  rewatch: false,
  watchCountText: "",
  venue: "theatre",
  format: null,
};

let refs: {
  title: HTMLInputElement;
  watched: HTMLInputElement;
  notes: HTMLTextAreaElement;
  rating: HTMLSelectElement;
  rewatch: HTMLInputElement;
  watchCount: HTMLInputElement;
} | null = null;
let controls: FormatControls | null = null;

function capture(): void {
  if (refs) {
    edit.title = refs.title.value;
    edit.watchedLocal = refs.watched.value;
    edit.notes = refs.notes.value;
    edit.rating = refs.rating.value as "none" | Rating;
    edit.rewatch = refs.rewatch.checked;
    edit.watchCountText = refs.watchCount.value;
  }
  if (controls) edit.format = controls.get();
}

function renderAgain(): void {
  capture();
  const content = document.querySelector<HTMLElement>(".content");
  if (content) { content.replaceChildren(); renderEdit(content); }
}

export function renderEdit(content: HTMLElement): void {
  const id = getState().editId;
  const log = id != null ? getState().logs.find((l) => l.id === id) : undefined;
  if (!log) {
    content.append(el("p", { class: "empty" }, "Log not found. "));
    content.lastElementChild?.append(el("button", { class: "btn", onclick: () => go("logs") }, "Back"));
    return;
  }

  // Seed local state when entering a new log.
  if (edit.id !== log.id) {
    edit.id = log.id;
    edit.title = log.title;
    edit.watchedLocal = toLocalInput(log.watched_at);
    edit.notes = log.notes ?? "";
    edit.rating = log.rating ?? "none";
    edit.rewatch = log.rewatch;
    edit.watchCountText = log.watch_count != null ? String(log.watch_count) : "";
    edit.venue = log.venue;
    edit.format = log.format ?? defaultFormat(log.venue);
  }

  const title = el("input", { class: "text", value: edit.title });
  const watched = el("input", { type: "datetime-local", value: edit.watchedLocal });
  const notes = el("textarea", { class: "notes", rows: 4 }, edit.notes);
  const rating = el("select", {},
    el("option", { value: "none" }, "No rating"),
    el("option", { value: "up" }, "👍"),
    el("option", { value: "down" }, "👎"),
  );
  rating.value = edit.rating;
  const rewatch = el("input", { type: "checkbox", checked: edit.rewatch });
  const watchCount = el("input", { type: "number", min: 1, value: edit.watchCountText });
  refs = { title, watched, notes, rating, rewatch, watchCount };

  const venueTheatre = el("input", { type: "radio", name: "venue", checked: edit.venue === "theatre" });
  const venueHome = el("input", { type: "radio", name: "venue", checked: edit.venue === "home" });
  const setVenue = (v: Venue) => {
    capture();
    edit.venue = v;
    if (!venueMatches(v)) {
      controls = null;
      edit.format = defaultFormat(v);
    }
    renderAgain();
  };
  venueTheatre.addEventListener("change", () => setVenue("theatre"));
  venueHome.addEventListener("change", () => setVenue("home"));

  controls = formatControls(edit.venue, edit.format);
  const formatArea = el("div", { class: "format-area" }, controls.root);

  content.append(
    el("form", { class: "edit-form", onsubmit: (e: Event) => { e.preventDefault(); save(); } },
      el("div", { class: "field-row" }, el("label", {}, "Title"), title),
      el("div", { class: "field-row" }, el("label", {}, "Watched at"), watched),
      el("div", { class: "field-row" }, el("label", {}, "Notes"), notes),
      el("div", { class: "field-row" }, el("label", {}, "Rating"), rating),
      el("div", { class: "field-row" }, el("label", {}, "Watch count"), watchCount),
      el("div", { class: "field-row" }, el("label", {}, "Rewatch"), rewatch),
      el("div", { class: "venue-row" }, el("label", {}, venueTheatre, " Theatre"), el("label", {}, venueHome, " Home")),
      el("div", { class: "field-row" }, el("label", {}, "Viewing format")),
      formatArea,
      el("div", { class: "actions" },
        el("button", { class: "btn primary", type: "submit" }, "Save"),
        el("button", { class: "btn", type: "button", onclick: () => go("logs") }, "Cancel")),
    )
  );
}

function venueMatches(venue: Venue): boolean {
  return edit.format != null && (venue === "theatre" ? "theatre" in edit.format : "home" in edit.format);
}

async function save(): Promise<void> {
  capture();
  const id = edit.id;
  if (id == null) return;
  const log = getState().logs.find((l) => l.id === id);
  if (!log) return;
  const updated: WatchLog = {
    ...log,
    title: edit.title,
    watched_at: fromLocalInput(edit.watchedLocal),
    notes: edit.notes.trim() === "" ? null : edit.notes,
    rating: edit.rating === "none" ? null : edit.rating,
    rewatch: edit.rewatch,
    watch_count: edit.watchCountText.trim() === "" ? null : Number(edit.watchCountText),
    venue: edit.venue,
    format: edit.format,
  };
  try {
    await api.updateLog(updated);
    await reload();
    go("logs");
  } catch (e) {
    showError(e);
  }
}
