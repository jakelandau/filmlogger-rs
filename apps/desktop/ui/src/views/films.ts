// The film library: every stored film with its viewing summary, filterable
// by favorites and searchable by title.
import { el } from "../dom";
import { api } from "../api";
import { getState, go, reload, showError } from "../app";
import { formatDisplay } from "../time";
import type { Collection, FilmListEntry, MovieSummary, SearchPage } from "../types";

const IMAGE_BASE = "https://image.tmdb.org/t/p/w92";

type Tab = "all" | "seen" | "favorites" | "bucket" | "collections";
let tab: Tab = "all";
let search = "";
let tmdbResults: SearchPage | null = null;
let searchedQuery = "";
let searchBusy = false;
let addingTmdb: number | null = null;
const collectionCache = new Map<number, Collection>();
const collectionLoading = new Set<number>();
const collectionFailed = new Set<number>();

export function renderFilms(content: HTMLElement): void {
  const films = getState().films;
  content.append(renderControls());

  if (tab === "collections") {
    renderCollections(content, search);
    return;
  }

  const needle = search.trim().toLowerCase();
  const visible = films.filter((entry) => {
    const isListed = entry.viewings > 0 || entry.film.watchlisted;
    if (tab === "all" && !isListed) return false;
    if (tab === "seen" && entry.viewings === 0) return false;
    if (tab === "favorites" && (!entry.film.favorite || !isListed)) return false;
    if (tab === "bucket" && (!entry.film.watchlisted || entry.viewings > 0)) return false;
    if (needle === "") return true;
    return entry.film.title.toLowerCase().includes(needle);
  });

  const libraryByTmdb = new Map(
    films
      .filter((entry) => entry.film.tmdb_id != null)
      .map((entry) => [entry.film.tmdb_id as number, entry])
  );
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
    if (visible.length > 0) list.append(el("h3", {}, "In your library"));
  }

  if (visible.length > 0) {
    for (const entry of visible) list.append(renderRow(entry));
  } else if (tmdbResults == null) {
    list.append(
      el(
        "p",
        { class: "empty" },
        films.length === 0 ? "No films yet. " : "No films match this filter. ",
        films.length === 0
          ? el("button", { class: "link", onclick: () => go("add") }, "Add your first film")
          : ""
      )
    );
  } else {
    list.append(el("p", { class: "empty" }, "No local matches."));
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

async function addToBucketList(tmdbId: number): Promise<void> {
  if (addingTmdb != null) return;
  addingTmdb = tmdbId;
  renderAgain();
  try {
    await api.addToWatchlist(tmdbId);
    await reload();
  } catch (e) {
    showError(e);
  } finally {
    addingTmdb = null;
    renderAgain();
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
              void addToBucketList(result.id);
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
  for (const collection of visible) list.append(renderCollection(collection));
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

function renderCollection(collection: Collection): HTMLElement {
  const parts = [...collection.parts].sort(
    (a, b) =>
      (a.release_year ?? Number.MAX_SAFE_INTEGER) -
        (b.release_year ?? Number.MAX_SAFE_INTEGER) || a.title.localeCompare(b.title)
  );
  const library = new Map(
    getState()
      .films.filter((entry) => entry.film.tmdb_id != null)
      .map((entry) => [entry.film.tmdb_id as number, entry])
  );
  const seenCount = parts.filter((part) => (library.get(part.tmdb_id)?.viewings ?? 0) > 0).length;
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
      el("span", { class: "collection-status" + (seen ? " seen" : "") }, seen ? "Seen" : "Not seen")
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
          ? el("div", { class: "dim" }, `${seenCount} of ${parts.length} seen`)
          : ""
      )
    ),
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
    ["all", "All"],
    ["seen", "Seen"],
    ["favorites", "Favorites"],
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
  chips.append(
    el("input", {
      class: "search",
      type: "text",
      placeholder: "Search title…",
      value: search,
      oninput: (e: Event) => {
        search = (e.currentTarget as HTMLInputElement).value;
        if (search.trim() !== searchedQuery) tmdbResults = null;
        renderAgain();
      },
    })
  );
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
        el("span", { class: "dim" }, `${entry.viewings} viewing${entry.viewings === 1 ? "" : "s"}`),
        entry.last_watched_at
          ? el("span", { class: "dim" }, " · last " + formatDisplay(entry.last_watched_at))
          : ""
      )
    )
  );
}
