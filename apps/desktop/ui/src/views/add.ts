// Add a film: search TMDB, pick a result, confirm the film, choose venue +
// optional format, then log it. Local UI state lives in this module; the
// whole view is re-rendered on each change, so format selections are
// captured into `add.format` before any re-render.
import { el } from "../dom";
import { api } from "../api";
import { go, reload, showError } from "../app";
import { defaultFormat, formatControls } from "../format-ui";
import type { FormatControls } from "../format-ui";
import type { Format, Movie, MovieSummary, Venue } from "../types";

const IMAGE_BASE = "https://image.tmdb.org/t/p/w92";

/// Wizard stage: results list, film confirmation, or data entry.
type Stage = "search" | "confirm" | "details";

interface Selected {
  id: number;
  title: string;
  year: number | null;
  poster: string | null;
  imdbId: string | null;
}

interface AddState {
  query: string;
  results: MovieSummary[];
  total_pages: number;
  selected: Selected | null;
  stage: Stage;
  venue: Venue;
  format: Format | null;
}

const add: AddState = {
  query: "",
  results: [],
  total_pages: 0,
  selected: null,
  stage: "search",
  venue: "theatre",
  format: null,
};

let controls: FormatControls | null = null;

// Preserve the live format selection across a re-render.
function captureFormat(): void {
  if (controls) add.format = controls.get();
}

export function renderAdd(content: HTMLElement): void {
  content.append(renderSearch());
  content.append(renderImdbFinder());
  const s = add.selected;
  if (s && add.stage === "confirm") {
    content.append(renderConfirm());
  } else if (s && add.stage === "details") {
    content.append(renderSelected());
  } else {
    content.append(renderResults());
  }
}

function renderSearch(): HTMLElement {
  const input = el("input", {
    class: "search",
    type: "text",
    placeholder: "Search films…",
    value: add.query,
    oninput: (e: Event) => {
      add.query = (e.currentTarget as HTMLInputElement).value;
    },
  });
  const btn = el("button", { class: "btn" }, "Search");
  const form = el(
    "form",
    { class: "search-form", onsubmit: (e: Event) => { e.preventDefault(); doSearch(); } },
    input,
    btn
  );
  return el("div", { class: "search-bar" }, form);
}

function renderResults(): HTMLElement {
  const box = el("div", { class: "results" });
  if (add.results.length === 0) return box;
  const list = el("ul", { class: "result-list" });
  for (const r of add.results) {
    list.append(
      el(
        "li",
        { class: "result", onclick: () => select(r) },
        r.poster_path
          ? el("img", { class: "poster", src: IMAGE_BASE + r.poster_path, alt: "" })
          : el("div", { class: "poster placeholder" }),
        el("div", { class: "result-text" },
          r.title,
          r.release_date ? el("span", { class: "dim" }, ` (${r.release_date.slice(0, 4)})`) : ""
        )
      )
    );
  }
  box.append(list);
  return box;
}

// Step 2: confirm the picked film before entering details.
function renderConfirm(): HTMLElement {
  const s = add.selected as Selected;
  const poster = s.poster
    ? el("img", { class: "poster confirm", src: IMAGE_BASE + s.poster, alt: "" })
    : el("div", { class: "poster confirm placeholder" });
  const back = el(
    "button",
    {
      class: "btn",
      onclick: () => {
        add.selected = null;
        add.stage = "search";
        add.format = null;
        controls = null;
        renderAgain();
      },
    },
    "Back"
  );
  const confirm = el(
    "button",
    { class: "btn primary", onclick: () => { add.stage = "details"; renderAgain(); } },
    "Confirm film"
  );
  return el(
    "div",
    { class: "selected-card" },
    el(
      "div",
      { class: "confirm-head" },
      poster,
      el("h3", {}, s.title, s.year != null ? el("span", { class: "dim" }, ` (${s.year})`) : "")
    ),
    s.imdbId ? el("div", { class: "dim" }, s.imdbId) : "",
    el("p", {}, "Is this the film you watched?"),
    el("div", { class: "actions" }, back, confirm)
  );
}

function renderSelected(): HTMLElement {
  const s = add.selected as Selected;

  const venueTheatre = el("input", { type: "radio", name: "venue", value: "theatre", checked: add.venue === "theatre" });
  const venueHome = el("input", { type: "radio", name: "venue", value: "home", checked: add.venue === "home" });
  const setVenue = (v: Venue) => {
    captureFormat();
    add.venue = v;
    if (!add.format || !venueMatches(add.venue)) add.format = defaultFormat(v);
    renderAgain();
  };
  venueTheatre.addEventListener("change", () => setVenue("theatre"));
  venueHome.addEventListener("change", () => setVenue("home"));

  if (!add.format || !venueMatches(add.venue)) add.format = defaultFormat(add.venue);
  controls = formatControls(add.venue, add.format);
  const formatArea = el("div", { class: "format-area" }, controls.root);

  const addBtn = el("button", { class: "btn primary", onclick: () => doAdd() }, "Log this film");

  return el(
    "div",
    { class: "selected-card" },
    el("h3", {}, s.title, s.year != null ? el("span", { class: "dim" }, ` (${s.year})`) : ""),
    el(
      "div",
      { class: "venue-row" },
      el("label", {}, venueTheatre, " Theatre"),
      el("label", {}, venueHome, " Home")
    ),
    el("div", { class: "field-row" }, el("label", {}, "Viewing format")),
    formatArea,
    el("div", { class: "actions" }, addBtn, el("button", { class: "btn", onclick: () => { add.stage = "confirm"; renderAgain(); } }, "Back"))
  );
}

function venueMatches(venue: Venue): boolean {
  return add.format != null && (venue === "theatre" ? "theatre" in add.format : "home" in add.format);
}

function renderAgain(): void {
  captureFormat();
  const content = document.querySelector<HTMLElement>(".content");
  if (content) { content.replaceChildren(); renderAdd(content); }
}

async function doSearch(): Promise<void> {
  const query = add.query.trim();
  if (query === "") return;
  try {
    const page = await api.searchMovies(query, 1);
    add.results = page.results;
    add.total_pages = page.total_pages;
    renderAgain();
  } catch (e) {
    showError(e);
  }
}

function select(r: MovieSummary | Movie): void {
  add.selected = {
    id: r.id,
    title: r.title,
    year: r.release_date ? Number(r.release_date.slice(0, 4)) : null,
    poster: r.poster_path,
    imdbId: "imdb_id" in r ? r.imdb_id : null,
  };
  add.stage = "confirm";
  add.format = null;
  controls = null;
  renderAgain();
}

async function doAdd(): Promise<void> {
  const s = add.selected as Selected;
  captureFormat();
  const format = add.format;
  try {
    await api.addFilm(s.id, add.venue, format);
    add.selected = null;
    add.stage = "search";
    add.results = [];
    add.format = null;
    controls = null;
    await reload();
    go("logs");
  } catch (e) {
    showError(e);
  }
}

// Alternate lookup: resolve a film directly from its IMDB id.
function renderImdbFinder(): HTMLElement {
  const input = el("input", { class: "search", type: "text", placeholder: "IMDB id (tt…)" });
  const btn = el("button", {
    class: "btn",
    onclick: async () => {
      const id = input.value.trim();
      if (id === "") return;
      try {
        const movie: Movie = await api.findImdb(id);
        select(movie);
      } catch (e) {
        showError(e);
      }
    },
  }, "Find by IMDB");
  return el("div", { class: "imdb-finder" }, input, btn);
}
