// The film library: every stored film with its viewing summary, filterable
// by favorites and searchable by title or tag.
import { el } from "../dom";
import { getState, go } from "../app";
import { formatDisplay } from "../time";
import type { FilmListEntry } from "../types";

const IMAGE_BASE = "https://image.tmdb.org/t/p/w92";

type Tab = "all" | "favorites";
let tab: Tab = "all";
let search = "";

export function renderFilms(content: HTMLElement): void {
  const films = getState().films;
  content.append(renderControls());

  const needle = search.trim().toLowerCase();
  const visible = films.filter((entry) => {
    if (tab === "favorites" && !entry.film.favorite) return false;
    if (needle === "") return true;
    return (
      entry.film.title.toLowerCase().includes(needle) ||
      entry.film.tags.some((tag) => tag.toLowerCase().includes(needle))
    );
  });

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
  if (visible.length === 0) {
    content.append(el("p", { class: "empty" }, "No films match this filter."));
    return;
  }

  const list = el("div", { class: "film-grid" });
  for (const entry of visible) list.append(renderRow(entry));
  content.append(list);
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
      placeholder: "Search title or tag…",
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

  const tags = el(
    "span",
    { class: "film-tags" },
    ...film.tags.map((tag) => el("span", { class: "tag" }, tag))
  );

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
      ),
      tags
    )
  );
}
