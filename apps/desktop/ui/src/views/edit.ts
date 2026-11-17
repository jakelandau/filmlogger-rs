// Edit a stored log: title, venue, format, watched-at, rating.
// Editable fields are captured into module state before any re-render so
// imperative widgets survive the full re-render. `rewatch` and `watch_count`
// are store-derived projections, shown read-only.
import { el } from "../dom";
import { api } from "../api";
import { getState, go, reload, showError } from "../app";
import { defaultFormat, formatControls } from "../format-ui";
import type { FormatControls } from "../format-ui";
import {
  fromLocalInput,
  systemTimeZone,
  timeZoneOptions,
  toLocalInput,
} from "../time";
import type { Format, Rating, Venue, WatchLog } from "../types";

interface EditState {
  id: number | null;
  title: string;
  watchedLocal: string;
  timezone: string;
  rating: "" | Rating;
  venue: Venue;
  format: Format | null;
}

const edit: EditState = {
  id: null,
  title: "",
  watchedLocal: "",
  timezone: systemTimeZone(),
  rating: "",
  venue: "theatre",
  format: null,
};

let refs: {
  title: HTMLInputElement;
  watched: HTMLInputElement;
  timezone: HTMLInputElement;
  rating: HTMLSelectElement;
} | null = null;
let controls: FormatControls | null = null;

function capture(): void {
  if (refs) {
    edit.title = refs.title.value;
    edit.watchedLocal = refs.watched.value;
    edit.timezone = refs.timezone.value;
    edit.rating = refs.rating.value as "" | Rating;
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
    edit.timezone = log.timezone ?? systemTimeZone();
    edit.watchedLocal = toLocalInput(log.watched_at, edit.timezone);
    edit.rating = log.rating ?? "";
    edit.venue = log.venue;
    edit.format = log.format ?? defaultFormat(log.venue);
  }

  const title = el("input", { class: "text", value: edit.title });
  const timezone = el("input", {
    class: "text",
    type: "text",
    list: "timezone-options",
    placeholder: "Search time zones…",
    value: edit.timezone,
  });
  const timezoneList = el(
    "datalist",
    { id: "timezone-options" },
    ...timeZoneOptions(edit.timezone).map((zone) => el("option", { value: zone }))
  );
  timezone.addEventListener("change", renderAgain);
  const watched = el("input", { type: "datetime-local", value: edit.watchedLocal });
  const rating = el("select", {},
    el("option", { value: "", disabled: true }, "Choose a rating"),
    el("option", { value: "up" }, "👍 Up"),
    el("option", { value: "down" }, "👎 Down"),
  );
  rating.value = edit.rating;
  rating.addEventListener("change", renderAgain);
  refs = { title, watched, timezone, rating };

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
      el("div", { class: "field-row" }, el("label", {}, "Time zone"), timezone, timezoneList),
      el("div", { class: "field-row" }, el("label", {}, "Watched at"), watched),
      el("div", { class: "field-row" }, el("label", {}, "Rating"), rating),
      el(
        "div",
        { class: "dim" },
        `Viewing ${log.watch_count ?? 1} of this film${log.rewatch ? " · rewatch" : ""}`
      ),
      el("div", { class: "venue-row" }, el("label", {}, venueTheatre, " Theatre"), el("label", {}, venueHome, " Home")),
      el("div", { class: "field-row" }, el("label", {}, "Viewing format")),
      formatArea,
      el("div", { class: "actions" },
        el("button", { class: "btn primary", type: "submit", disabled: edit.rating === "" }, "Save"),
        el("button", { class: "btn", type: "button", onclick: () => go("logs") }, "Cancel")),
    )
  );
  content.append(
    el(
      "div",
      { class: "actions danger-zone" },
      el(
        "button",
        { class: "btn danger", type: "button", onclick: () => void deleteCurrent() },
        "Delete viewing…"
      )
    )
  );
}

async function deleteCurrent(): Promise<void> {
  const id = edit.id;
  if (id == null) return;
  const log = getState().logs.find((l) => l.id === id);
  if (!log) return;
  if (!confirm(`Delete “${log.title}”? This cannot be undone.`)) return;
  try {
    await api.deleteLog(id);
    await reload();
    go("logs");
  } catch (e) {
    showError(e);
  }
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
  const rating = edit.rating;
  if (rating === "") {
    showError(new Error("Choose a rating"));
    return;
  }
  const watchedAt = fromLocalInput(edit.watchedLocal, edit.timezone);
  if (watchedAt === "") {
    showError(new Error("Invalid watched-at date or time zone"));
    return;
  }
  const updated: WatchLog = {
    ...log,
    title: edit.title,
    watched_at: watchedAt,
    timezone: edit.timezone,
    rating,
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
