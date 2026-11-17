// The film library: every stored film with its viewing summary, filterable
// by favorites and searchable by title.
import { el } from "../dom";
import { api } from "../api";
import { getState, go, reload, showError } from "../app";
import { formatDisplay } from "../time";
import {
  collectionMissingParts,
  collectionSeenCount,
  filmLibraryByTmdb,
  isCollectionComplete,
  visibleLibraryFilms,
} from "../film-sections";
import type { Collection, FilmListEntry, MovieSummary, SearchPage } from "../types";

const IMAGE_BASE = "https://image.tmdb.org/t/p/w92";

type Tab = "seen" | "bucket" | "collections";
let tab: Tab = "seen";
let favoritesOnly = false;
let search = "";
let tmdbResults: SearchPage | null = null;
let searchedQuery = "";
let searchBusy = false;
let addingTmdb: number | null = null;
let addingCollection: number | null = null;
const collectionCache = new Map<number, Collection>();
const collectionLoading = new Set<number>();
const collectionFailed = new Set<number>();

/**
 * Drop every cached TMDB lookup the Films tab holds. An import replaces the
 * whole database, so collection membership resolved against the old one is
 * no longer trustworthy.
 */
export function clearFilmCaches(): void {
  collectionCache.clear();
  collectionLoading.clear();
  collectionFailed.clear();
  tmdbResults = null;
  searchedQuery = "";
}

export function renderFilms(content: HTMLElement): void {
  const films = getState().films;
  content.append(renderControls());

  if (tab === "collections") {
    renderCollections(content, search);
    return;
  }

  const needle = search.trim().toLowerCase();
  const visible = visibleLibraryFilms(films, tab, favoritesOnly, needle);
  const sectionLabel = tab === "seen" ? "Seen" : "Bucket List";
  const libraryByTmdb = filmLibraryByTmdb(films);
  const list = el("div", { class: "film-grid" });

  if (tmdbResults != null) {
    list.append(el("h3", {}, `TMDB results for “${searchedQuery}”`));
    if (tmdbResults.results.length === 0) {
      list.append(el("p", { class: "empty" }, "No TMDB results."));
    }
    for (const result of tmdbResults.results) {
      list.append(renderTmdbResult(result, libraryByTmdb.get(result.id)));
    }
    if (tmdbResults.page < tmdbResults.total_pages) {
      list.append(
        el(
          "button",
          {
            class: "btn load-more",
            disabled: searchBusy,
            onclick: () => void searchTmdb(tmdbResults!.page + 1),
          },
          searchBusy ? "Loading…" : "Load more"
        )
      );
    }
    if (visible.length > 0) list.append(el("h3", {}, sectionLabel));
  }

  if (visible.length > 0) {
    for (const entry of visible) list.append(renderRow(entry));
  } else if (films.length === 0) {
    list.append(
      el("h3", {}, "No films yet"),
      el("p", { class: "empty" }, "Search TMDB or add a film from the Add tab."),
      el("button", { class: "btn", onclick: () => go("add") }, "Add your first film")
    );
  } else if (tmdbResults != null) {
    list.append(el("p", { class: "empty" }, "No local matches."));
  } else if (needle !== "") {
    list.append(el("p", { class: "empty" }, "No films match this filter."));
  } else if (tab === "seen" && favoritesOnly) {
    list.append(el("p", { class: "empty" }, "No favorite films seen."));
  } else if (tab === "seen") {
    list.append(el("p", { class: "empty" }, "No films seen yet."));
  } else {
    list.append(
      el(
        "p",
        { class: "empty" },
        "Your bucket list is empty. Add films from TMDB search or from a collection."
      )
    );
  }

  content.append(list);
}

async function searchTmdb(page = 1): Promise<void> {
  const query = search.trim();
  if (query === "") return;
  searchBusy = true;
  renderAgain();
  try {
    const next = await api.searchMovies(query, page);
    tmdbResults =
      page === 1
        ? next
        : {
            ...next,
            results: [...(tmdbResults?.results ?? []), ...next.results],
          };
    searchedQuery = query;
  } catch (e) {
    showError(e);
  } finally {
    searchBusy = false;
    renderAgain();
  }
}

async function addToBucketList(tmdbId: number, entry?: FilmListEntry): Promise<void> {
  if (addingTmdb != null || addingCollection != null) return;
  addingTmdb = tmdbId;
  renderAgain();
  try {
    if (entry != null) await api.setWatchlisted(entry.film.film_id, true);
    else await api.addToWatchlist(tmdbId);
    await reload();
  } catch (e) {
    showError(e);
  } finally {
    addingTmdb = null;
    renderAgain();
  }
}

// Add every part that is neither seen nor watchlisted. Individual failures
// do not abort the run; the count is reported once at the end.
async function addMissingToBucketList(collection: Collection): Promise<void> {
  if (addingTmdb != null || addingCollection != null) return;
  addingCollection = collection.id;
  renderAgain();
  const library = filmLibraryByTmdb(getState().films);
  const missing = collectionMissingParts(collection, library);
  let failures = 0;
  try {
    for (const part of missing) {
      const entry = library.get(part.tmdb_id);
      try {
        if (entry != null) await api.setWatchlisted(entry.film.film_id, true);
        else await api.addToWatchlist(part.tmdb_id);
      } catch {
        failures += 1;
      }
    }
    await reload();
  } catch (e) {
    showError(e);
  } finally {
    addingCollection = null;
    renderAgain();
  }
  if (failures > 0) {
    showError(`${failures} of ${missing.length} films could not be added to the bucket list`);
  }
}

function renderTmdbResult(result: MovieSummary, entry: FilmListEntry | undefined): HTMLElement {
  const seen = (entry?.viewings ?? 0) > 0;
  const onBucketList = entry?.film.watchlisted === true;
  const status = seen
    ? "Seen"
    : onBucketList
    ? "On Bucket List"
    : entry != null
    ? "In library"
    : "Not in library";
  const poster = result.poster_path
    ? el("img", { class: "poster", src: IMAGE_BASE + result.poster_path, alt: "" })
    : el("div", { class: "poster placeholder" });
  const row = el(
    "div",
    { class: "film-row tmdb-result" + (entry != null ? " clickable" : "") },
    poster,
    el(
      "div",
      { class: "film-main" },
      el(
        "div",
        { class: "film-title" },
        result.title,
        result.release_date != null
          ? el("span", { class: "dim" }, ` (${result.release_date.slice(0, 4)})`)
          : ""
      ),
      el("div", { class: "log-sub" }, status)
    ),
    !seen && !onBucketList
      ? el(
          "button",
          {
            class: "btn",
            disabled: addingTmdb === result.id,
            onclick: (e: Event) => {
              e.stopPropagation();
              void addToBucketList(result.id, entry);
            },
          },
          addingTmdb === result.id ? "Adding…" : "Bucket List"
        )
      : ""
  );
  if (entry != null) {
    row.addEventListener("click", () => go("film", entry.film.film_id));
  }
  return row;
}

function ensureCollection(id: number): void {
  if (
    collectionCache.has(id) ||
    collectionLoading.has(id) ||
    collectionFailed.has(id)
  ) {
    return;
  }
  collectionLoading.add(id);
  void api
    .getCollection(id)
    .then((collection) => {
      collectionCache.set(id, collection);
      collectionLoading.delete(id);
      renderAgain();
    })
    .catch(() => {
      collectionLoading.delete(id);
      collectionFailed.add(id);
      renderAgain();
    });
}

function renderCollections(content: HTMLElement, query: string): void {
  const refs = new Map<number, string>();
  for (const entry of getState().films) {
    const collection = entry.film.collection;
    if (collection != null && !refs.has(collection.id)) {
      refs.set(collection.id, collection.name);
      ensureCollection(collection.id);
    }
  }

  const needle = query.trim().toLowerCase();
  const all = [...refs.keys()]
    .map((id) => collectionCache.get(id))
    .filter((collection): collection is Collection => collection != null);
  const visible = all.filter(
    (collection) =>
      needle === "" ||
      collection.name.toLowerCase().includes(needle) ||
      collection.parts.some((part) => part.title.toLowerCase().includes(needle))
  );

  if (all.length === 0) {
    content.append(
      el(
        "p",
        { class: "empty" },
        collectionLoading.size > 0 ? "Loading collections…" : "No films in a collection yet."
      )
    );
    return;
  }
  if (visible.length === 0) {
    content.append(el("p", { class: "empty" }, "No collections match this search."));
    return;
  }

  const list = el("div", { class: "film-grid" });
  const library = filmLibraryByTmdb(getState().films);
  if (needle === "") {
    const completed = all.filter((collection) => isCollectionComplete(collection, library)).length;
    content.append(el("p", { class: "dim" }, `${completed} of ${all.length} collections completed`));
  }
  for (const collection of [...visible].sort(
    (a, b) =>
      Number(isCollectionComplete(a, library)) - Number(isCollectionComplete(b, library)) ||
      a.name.localeCompare(b.name, undefined, { sensitivity: "base" })
  )) {
    list.append(renderCollection(collection, library));
  }
  content.append(list);
}

async function openCollectionPart(tmdbId: number): Promise<void> {
  try {
    const film = await api.ensureFilm(tmdbId);
    await reload();
    go("film", film.film_id);
  } catch (e) {
    showError(e);
  }
}

function renderCollection(
  collection: Collection,
  library: Map<number, FilmListEntry>
): HTMLElement {
  const parts = [...collection.parts].sort(
    (a, b) =>
      (a.release_year ?? Number.MAX_SAFE_INTEGER) -
        (b.release_year ?? Number.MAX_SAFE_INTEGER) || a.title.localeCompare(b.title)
  );
  const seenCount = collectionSeenCount(collection, library);
  const complete = isCollectionComplete(collection, library);
  const missing = collectionMissingParts(collection, library);
  const percent = parts.length > 0 ? Math.round((seenCount / parts.length) * 100) : 0;
  const busy = addingTmdb != null || addingCollection != null;
  const posterPath = collection.poster_path ?? parts[0]?.poster_path ?? null;
  const poster = posterPath
    ? el("img", { class: "poster", src: IMAGE_BASE + posterPath, alt: "" })
    : el("div", { class: "poster placeholder" });

  const rows = el("div", { class: "collection-parts" });
  if (parts.length === 0) {
    rows.append(el("p", { class: "empty" }, "No films returned for this collection."));
  }
  for (const part of parts) {
    const entry = library.get(part.tmdb_id);
    const seen = (entry?.viewings ?? 0) > 0;
    const status = seen
      ? el("span", { class: "collection-status seen" }, "Seen")
      : entry?.film.watchlisted
      ? el("span", { class: "collection-status bucket" }, "On Bucket List")
      : el(
          "button",
          {
            class: "btn",
            type: "button",
            disabled: busy,
            onclick: (event: Event) => {
              event.stopPropagation();
              void addToBucketList(part.tmdb_id, entry);
            },
          },
          addingTmdb === part.tmdb_id ? "Adding…" : "Bucket List"
        );
    const row = el(
      "div",
      { class: "collection-part clickable" },
      el("span", { class: "collection-mark" + (seen ? " seen" : "") }, seen ? "✓" : "○"),
      el(
        "span",
        { class: "collection-part-title" },
        part.title,
        part.release_year != null ? el("span", { class: "dim" }, ` (${part.release_year})`) : ""
      ),
      status
    );
    row.addEventListener("click", () => {
      if (entry != null) {
        go("film", entry.film.film_id);
      } else {
        void openCollectionPart(part.tmdb_id);
      }
    });
    rows.append(row);
  }

  return el(
    "div",
    { class: "collection-card" },
    el(
      "div",
      { class: "collection-head" },
      poster,
      el(
        "div",
        { class: "collection-main" },
        el("div", { class: "collection-title" }, collection.name),
        parts.length > 0
          ? el(
              "div",
              { class: "dim" },
              `${seenCount} of ${parts.length} seen`,
              complete ? el("span", { class: "collection-badge" }, "Completed") : ""
            )
          : ""
      ),
      parts.length > 0
        ? el(
            "div",
            { class: "collection-ring", style: `--pct:${percent}` },
            el("span", {}, `${percent}%`)
          )
        : ""
    ),
    missing.length > 0
      ? el(
          "div",
          { class: "collection-actions" },
          el(
            "button",
            {
              class: "btn",
              type: "button",
              disabled: busy,
              onclick: (event: Event) => {
                event.stopPropagation();
                void addMissingToBucketList(collection);
              },
            },
            addingCollection === collection.id ? "Adding…" : "Add missing to Bucket List"
          )
        )
      : "",
    rows
  );
}

function renderAgain(): void {
  const content = document.querySelector<HTMLElement>(".content");
  if (content) {
    content.replaceChildren();
    renderFilms(content);
  }
}

function renderControls(): HTMLElement {
  const tabs: [Tab, string][] = [
    ["seen", "Seen"],
    ["bucket", "Bucket List"],
    ["collections", "Collections"],
  ];
  const chips = el("div", { class: "filter-bar" });
  for (const [value, label] of tabs) {
    chips.append(
      el(
        "button",
        {
          class: tab === value ? "chip active" : "chip",
          onclick: () => {
            tab = value;
            renderAgain();
          },
        },
        label
      )
    );
  }
  if (tab === "seen") {
    chips.append(
      el(
        "button",
        {
          class: favoritesOnly ? "chip active" : "chip",
          onclick: () => {
            favoritesOnly = !favoritesOnly;
            renderAgain();
          },
        },
        "Favorites"
      )
    );
  }
  chips.append(
    el("input", {
      class: "search",
      type: "text",
      placeholder: tab === "collections" ? "Search collections" : "Search films",
      value: search,
      oninput: (e: Event) => {
        search = (e.currentTarget as HTMLInputElement).value;
        if (search.trim() !== searchedQuery) tmdbResults = null;
        renderAgain();
        // The list re-render replaces the field, so typing would stop after one
        // character unless focus and the caret are restored.
        const field = document.querySelector<HTMLInputElement>(".search");
        if (field) {
          field.focus();
          field.setSelectionRange(field.value.length, field.value.length);
        }
      },
    })
  );
  if (tab !== "collections") {
    chips.append(
      el(
        "button",
        {
          class: "btn primary",
          type: "button",
          disabled: searchBusy || search.trim() === "",
          onclick: () => void searchTmdb(1),
        },
        searchBusy ? "Searching…" : "Search TMDB"
      )
    );
  }
  return chips;
}

function renderRow(entry: FilmListEntry): HTMLElement {
  const film = entry.film;
  const poster = film.poster_path
    ? el("img", { class: "poster", src: IMAGE_BASE + film.poster_path, alt: "" })
    : el("div", { class: "poster placeholder" });

  return el(
    "div",
    { class: "film-row", onclick: () => go("film", film.film_id) },
    poster,
    el(
      "div",
      { class: "film-main" },
      el(
        "div",
        { class: "film-title" },
        film.title,
        film.release_year != null ? el("span", { class: "dim" }, ` (${film.release_year})`) : "",
        film.favorite ? el("span", { class: "badge rewatch" }, " ★") : ""
      ),
      el(
        "div",
        { class: "log-sub" },
        entry.viewings === 0
          ? el("span", { class: "bucket-status" }, "On Bucket List")
          : el("span", { class: "dim" }, `${entry.viewings} viewing${entry.viewings === 1 ? "" : "s"}`),
        entry.last_watched_at
          ? el("span", { class: "dim" }, " · last " + formatDisplay(entry.last_watched_at))
          : ""
      )
    )
  );
}
