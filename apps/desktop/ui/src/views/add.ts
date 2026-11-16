// Add a film: search TMDB (or pick a popular title), confirm the film,
// choose venue + format and the viewing details, then log it. The same
// wizard also logs another viewing of an already-stored film.
import { el } from "../dom";
import { api } from "../api";
import { go, reload, showError } from "../app";
import { defaultFormat, formatControls } from "../format-ui";
import type { FormatControls } from "../format-ui";
import {
  fromLocalInput,
  systemTimeZone,
  timeZoneOptions,
  toLocalInput,
} from "../time";
import type {
  FilmMetadata,
  Format,
  Movie,
  MovieSummary,
  NewLogDetails,
  Rating,
  Venue,
} from "../types";

const IMAGE_BASE = "https://image.tmdb.org/t/p/w92";

/// Wizard stage: results list, film confirmation, venue/format, or viewing details.
type Stage = "search" | "confirm" | "format" | "details";

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
  page: number;
  totalPages: number;
  loading: boolean;
  // Popular films, shown before any search so the list isn't empty.
  popular: MovieSummary[];
  popularPage: number;
  popularTotalPages: number;
  popularLoading: boolean;
  // True once a title search has run; the list then shows search results.
  searched: boolean;
  selected: Selected | null;
  stage: Stage;
  mode: "add" | "log_film";
  filmId: string | null;
  venue: Venue;
  format: Format | null;
  watchedLocal: string;
  timezone: string;
  rating: "" | Rating;
}

const SYSTEM_ZONE = systemTimeZone();
const nowLocal = (timeZone: string) =>
  toLocalInput(new Date().toISOString(), timeZone);

const add: AddState = {
  query: "",
  results: [],
  page: 0,
  totalPages: 0,
  loading: false,
  popular: [],
  popularPage: 0,
  popularTotalPages: 0,
  popularLoading: false,
  searched: false,
  selected: null,
  stage: "search",
  mode: "add",
  filmId: null,
  venue: "theatre",
  format: null,
  watchedLocal: nowLocal(SYSTEM_ZONE),
  timezone: SYSTEM_ZONE,
  rating: "",
};

let controls: FormatControls | null = null;
let refs: {
  watched: HTMLInputElement;
  timezone: HTMLInputElement;
  rating: HTMLSelectElement;
} | null = null;

// Preserve the live format/details selection across a re-render.
function captureDetails(): void {
  if (controls) add.format = controls.get();
  if (refs) {
    add.watchedLocal = refs.watched.value;
    add.timezone = refs.timezone.value;
    add.rating = refs.rating.value as "" | Rating;
  }
}

function resetAdd(): void {
  add.query = "";
  add.results = [];
  add.page = 0;
  add.totalPages = 0;
  add.searched = false;
  add.selected = null;
  add.stage = "search";
  add.mode = "add";
  add.filmId = null;
  add.format = null;
  add.timezone = SYSTEM_ZONE;
  add.watchedLocal = nowLocal(add.timezone);
  add.rating = "";
  controls = null;
  refs = null;
}

/** Start the wizard already on a stored film, to log another viewing. */
export function startAddForFilm(film: FilmMetadata): void {
  add.selected = {
    id: film.tmdb_id ?? 0,
    title: film.title,
    year: film.release_year,
    poster: film.poster_path,
    imdbId: film.imdb_id,
  };
  add.mode = "log_film";
  add.filmId = film.film_id;
  add.stage = "format";
  add.venue = "theatre";
  add.format = defaultFormat("theatre");
  add.timezone = SYSTEM_ZONE;
  add.watchedLocal = nowLocal(add.timezone);
  add.rating = "";
  controls = null;
  refs = null;
  go("add");
}

export function renderAdd(content: HTMLElement): void {
  const s = add.selected;
  if (s === null) {
    // Search stage: title search + IMDB finder, with the results below.
    // The confirm and data-entry stages drop the search fields.
    content.append(renderSearch());
    content.append(renderImdbFinder());
    content.append(renderResults());
    ensurePopular();
  } else if (add.stage === "confirm") {
    content.append(renderConfirm());
  } else if (add.stage === "format") {
    content.append(renderFormat());
  } else {
    content.append(renderDetails());
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
    { class: "search-form", onsubmit: (e: Event) => { e.preventDefault(); void doSearch(); } },
    input,
    btn
  );
  return el("div", { class: "search-bar" }, form);
}

// Fetch and cache the popular films once, before any search runs, so the
// list shows something rather than empty space.
let popularStarted = false;
function ensurePopular(): void {
  if (popularStarted || add.searched) return;
  popularStarted = true;
  add.popularLoading = true;
  api
    .popularMovies(1)
    .then((page) => {
      add.popular = page.results;
      add.popularPage = page.page;
      add.popularTotalPages = page.total_pages;
      add.popularLoading = false;
      renderAgain();
    })
    .catch((e) => {
      add.popularLoading = false;
      showError(e);
    });
}

function renderResults(): HTMLElement {
  const box = el("div", { class: "results" });
  // Before a search is entered, show the popular films so the list isn't
  // empty; once a search has run, show its results instead.
  const list = add.searched ? add.results : add.popular;
  if (!add.searched) {
    box.append(el("p", { class: "results-label" }, "Popular right now"));
    if (add.popularLoading) {
      box.append(el("p", { class: "empty" }, "Loading…"));
      return box;
    }
  }
  if (list.length === 0) return box;
  const ul = el("ul", { class: "result-list" });
  for (const r of list) ul.append(renderResultRow(r));
  box.append(ul);

  const more = add.searched
    ? add.page < add.totalPages
    : add.popularPage < add.popularTotalPages;
  if (more) {
    box.append(
      el(
        "button",
        {
          class: "btn load-more",
          disabled: add.searched ? add.loading : add.popularLoading,
          onclick: () => void (add.searched ? loadMoreSearch() : loadMorePopular()),
        },
        "Load more"
      )
    );
  }
  return box;
}

function renderResultRow(r: MovieSummary): HTMLElement {
  return el(
    "li",
    { class: "result", onclick: () => select(r) },
    r.poster_path
      ? el("img", { class: "poster", src: IMAGE_BASE + r.poster_path, alt: "" })
      : el("div", { class: "poster placeholder" }),
    el(
      "div",
      { class: "result-text" },
      r.title,
      r.release_date ? el("span", { class: "dim" }, ` (${r.release_date.slice(0, 4)})`) : ""
    )
  );
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
    { class: "btn primary", onclick: () => { add.stage = "format"; renderAgain(); } },
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

function renderFormat(): HTMLElement {
  const s = add.selected as Selected;
  refs = null;

  const venueTheatre = el("input", { type: "radio", name: "venue", value: "theatre", checked: add.venue === "theatre" });
  const venueHome = el("input", { type: "radio", name: "venue", value: "home", checked: add.venue === "home" });
  const setVenue = (v: Venue) => {
    captureDetails();
    add.venue = v;
    if (!add.format || !venueMatches(add.venue)) add.format = defaultFormat(v);
    renderAgain();
  };
  venueTheatre.addEventListener("change", () => setVenue("theatre"));
  venueHome.addEventListener("change", () => setVenue("home"));

  if (!add.format || !venueMatches(add.venue)) add.format = defaultFormat(add.venue);
  controls = formatControls(add.venue, add.format);
  const formatArea = el("div", { class: "format-area" }, controls.root);

  const next = el(
    "button",
    {
      class: "btn primary",
      onclick: () => {
        captureDetails();
        add.stage = "details";
        renderAgain();
      },
    },
    "Next"
  );
  const back = el(
    "button",
    {
      class: "btn",
      onclick: () => {
        captureDetails();
        if (add.mode === "log_film") {
          add.selected = null;
          add.mode = "add";
          add.filmId = null;
          add.stage = "search";
          add.format = null;
        } else {
          add.stage = "confirm";
        }
        controls = null;
        refs = null;
        renderAgain();
      },
    },
    "Back"
  );

  const poster = s.poster
    ? el("img", { class: "poster details", src: IMAGE_BASE + s.poster, alt: "" })
    : el("div", { class: "poster details placeholder" });
  return el(
    "div",
    { class: "selected-card" },
    el(
      "div",
      { class: "details-row" },
      el(
        "div",
        { class: "details-form" },
        el("h3", {}, s.title, s.year != null ? el("span", { class: "dim" }, ` (${s.year})`) : ""),
        el("p", { class: "dim" }, "Step 1 of 2 · Venue and format"),
        el(
          "div",
          { class: "venue-row" },
          el("label", {}, venueTheatre, " Theatre"),
          el("label", {}, venueHome, " Home")
        ),
        el("div", { class: "field-row" }, el("label", {}, "Viewing format")),
        formatArea,
        el("div", { class: "actions" }, next, back)
      ),
      poster
    )
  );
}

function renderDetails(): HTMLElement {
  const s = add.selected as Selected;
  controls = null;

  const timezone = el("input", {
    class: "text",
    type: "text",
    list: "timezone-options",
    placeholder: "Search time zones…",
    value: add.timezone,
  });
  const timezoneList = el(
    "datalist",
    { id: "timezone-options" },
    ...timeZoneOptions(add.timezone).map((zone) => el("option", { value: zone }))
  );
  timezone.addEventListener("change", () => {
    captureDetails();
    renderAgain();
  });
  const watched = el("input", { type: "datetime-local", value: add.watchedLocal });
  const nowBtn = el(
    "button",
    {
      class: "btn",
      type: "button",
      onclick: () => {
        captureDetails();
        add.watchedLocal = nowLocal(add.timezone);
        renderAgain();
      },
    },
    "Now"
  );
  const rating = el(
    "select",
    {},
    el("option", { value: "", disabled: true }, "Choose a rating"),
    el("option", { value: "up" }, "👍"),
    el("option", { value: "down" }, "👎")
  );
  rating.value = add.rating;
  rating.addEventListener("change", () => {
    captureDetails();
    renderAgain();
  });
  refs = { watched, timezone, rating };

  const addBtn = el(
    "button",
    {
      class: "btn primary",
      disabled: add.rating === "",
      onclick: () => void doAdd(),
    },
    add.mode === "log_film" ? "Log this viewing" : "Log this film"
  );
  const back = el(
    "button",
    {
      class: "btn",
      onclick: () => {
        captureDetails();
        add.stage = "format";
        refs = null;
        renderAgain();
      },
    },
    "Back"
  );

  const poster = s.poster
    ? el("img", { class: "poster details", src: IMAGE_BASE + s.poster, alt: "" })
    : el("div", { class: "poster details placeholder" });
  return el(
    "div",
    { class: "selected-card" },
    el(
      "div",
      { class: "details-row" },
      el(
        "div",
        { class: "details-form" },
        el("h3", {}, s.title, s.year != null ? el("span", { class: "dim" }, ` (${s.year})`) : ""),
        el("p", { class: "dim" }, "Step 2 of 2 · Viewing details"),
        el("div", { class: "field-row" }, el("label", {}, "Time zone"), timezone, timezoneList),
        el("div", { class: "field-row" }, el("label", {}, "Watched at"), watched, nowBtn),
        el("div", { class: "field-row" }, el("label", {}, "Rating"), rating),
        el("div", { class: "actions" }, addBtn, back)
      ),
      poster
    )
  );
}

function venueMatches(venue: Venue): boolean {
  return add.format != null && (venue === "theatre" ? "theatre" in add.format : "home" in add.format);
}

function renderAgain(): void {
  captureDetails();
  const content = document.querySelector<HTMLElement>(".content");
  if (content) {
    content.replaceChildren();
    renderAdd(content);
  }
}

async function doSearch(): Promise<void> {
  const query = add.query.trim();
  if (query === "") return;
  try {
    const page = await api.searchMovies(query, 1);
    add.results = page.results;
    add.page = page.page;
    add.totalPages = page.total_pages;
    add.searched = true;
    renderAgain();
  } catch (e) {
    showError(e);
  }
}

async function loadMoreSearch(): Promise<void> {
  if (add.loading) return;
  add.loading = true;
  renderAgain();
  try {
    const page = await api.searchMovies(add.query.trim(), add.page + 1);
    add.results = [...add.results, ...page.results];
    add.page = page.page;
    add.totalPages = page.total_pages;
  } catch (e) {
    showError(e);
  } finally {
    add.loading = false;
    renderAgain();
  }
}

async function loadMorePopular(): Promise<void> {
  if (add.popularLoading) return;
  add.popularLoading = true;
  renderAgain();
  try {
    const page = await api.popularMovies(add.popularPage + 1);
    add.popular = [...add.popular, ...page.results];
    add.popularPage = page.page;
    add.popularTotalPages = page.total_pages;
  } catch (e) {
    showError(e);
  } finally {
    add.popularLoading = false;
    renderAgain();
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
  add.mode = "add";
  add.filmId = null;
  add.format = null;
  controls = null;
  renderAgain();
}

async function doAdd(): Promise<void> {
  captureDetails();
  const s = add.selected;
  if (s === null) return;
  const rating = add.rating;
  if (rating === "") {
    showError(new Error("Choose a rating"));
    return;
  }
  const watchedAt = fromLocalInput(add.watchedLocal, add.timezone);
  if (watchedAt === "") {
    showError(new Error("Invalid watched-at date or time zone"));
    return;
  }
  const details: NewLogDetails = {
    watched_at: watchedAt,
    timezone: add.timezone,
    notes: null,
    rating,
  };
  const format = add.format ?? defaultFormat(add.venue);
  try {
    if (add.mode === "log_film" && add.filmId !== null) {
      await api.logFilm(add.filmId, add.venue, format, details);
    } else {
      await api.addFilm(s.id, add.venue, format, details);
    }
    resetAdd();
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
