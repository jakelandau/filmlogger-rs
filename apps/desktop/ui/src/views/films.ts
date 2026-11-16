// The film library: every stored film with its viewing summary, filterable
// by favorites and searchable by title.
import { el } from "../dom";
import { api } from "../api";
import { getState, go } from "../app";
import { formatDisplay } from "../time";
import type { Collection, FilmListEntry } from "../types";

const IMAGE_BASE = "https://image.tmdb.org/t/p/w92";

type Tab = "all" | "favorites" | "collections";
let tab: Tab = "all";
let search = "";
const collectionCache = new Map<number, Collection>();
const collectionLoading = new Set<number>();
const collectionFailed = new Set<number>();

export function renderFilms(content: HTMLElement): void {
  const films = getState().films;
  content.append(renderControls());

  if (films.length === 0) {
    content.append(
      el(
        "p",
        { class: "empty" },
        "No films yet. ",
        el("button", { class: "link", onclick: () => go("add") }, "Add your first film")
      )
    );
    return;
  }

  if (tab === "collections") {
    renderCollections(content, search);
    return;
  }

  const needle = search.trim().toLowerCase();
  const visible = films.filter((entry) => {
    if (tab === "favorites" && !entry.film.favorite) return false;
    if (needle === "") return true;
    return entry.film.title.toLowerCase().includes(needle);
  });

  if (visible.length === 0) {
    content.append(el("p", { class: "empty" }, "No films match this filter."));
    return;
  }

  const list = el("div", { class: "film-grid" });
  for (const entry of visible) list.append(renderRow(entry));
  content.append(list);
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
      { class: "collection-part" + (entry ? " clickable" : "") },
      el("span", { class: "collection-mark" + (seen ? " seen" : "") }, seen ? "✓" : "○"),
      el(
        "span",
        { class: "collection-part-title" },
        part.title,
        part.release_year != null ? el("span", { class: "dim" }, ` (${part.release_year})`) : ""
      ),
      el("span", { class: "collection-status" + (seen ? " seen" : "") }, seen ? "Seen" : "Not seen")
    );
    if (entry != null) {
      row.addEventListener("click", () => go("film", entry.film.film_id));
    }
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
    ["favorites", "Favorites"],
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
        renderAgain();
      },
    })
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
