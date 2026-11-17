// One film's page: metadata, user state (favorite), metadata refresh,
// viewing statistics, and the viewing history.
import { el } from "../dom";
import { api } from "../api";
import { getState, go, reload, showError } from "../app";
import { formatDisplay } from "../time";
import { startAddForFilm } from "./add";
import { certificationIcon } from "../cert-icons";
import { formatFamilyBadge } from "../format-family";
import { formatFlagsRow } from "../format-flags";
import type { Certifications, FilmPage, FilmStats } from "../types";

const IMAGE_BASE = "https://image.tmdb.org/t/p/w342";

export function renderFilm(content: HTMLElement): void {
  const page = getState().filmPage;
  const filmId = getState().filmId;
  if (page === null || filmId === null) {
    content.append(el("p", { class: "empty" }, "Loading…"));
    return;
  }
  content.append(
    el(
      "div",
      { class: "film-page" },
      renderHeader(page),
      renderActions(page),
      renderStats(page),
      renderHistory(page)
    )
  );
}

function refresh(): void {
  const filmId = getState().filmId;
  if (filmId !== null) go("film", filmId);
}

async function mutate(action: () => Promise<unknown>): Promise<void> {
  try {
    await action();
    await reload();
    refresh();
  } catch (e) {
    showError(e);
  }
}

function renderHeader(page: FilmPage): HTMLElement {
  const film = page.film;
  const poster = film.poster_path
    ? el("img", { class: "poster film-poster", src: IMAGE_BASE + film.poster_path, alt: "" })
    : el("div", { class: "poster film-poster placeholder" });

  const certs = certificationsLine(film.certifications, getState().settings.selected_board);
  const meta: string[] = [];
  if (film.release_year != null) meta.push(String(film.release_year));
  if (film.runtime_minutes != null) meta.push(`${film.runtime_minutes} min`);
  if (film.genres.length > 0) meta.push(film.genres.map((g) => g.name).join(", "));

  return el(
    "div",
    { class: "film-meta" },
    poster,
    el(
      "div",
      { class: "film-meta-text" },
      el(
        "h2",
        {},
        el("button", { class: "link back", onclick: () => go("films") }, "← Films "),
        film.title
      ),
      meta.length > 0 ? el("div", { class: "dim" }, meta.join(" · ")) : "",
      certs ?? "",
      film.tagline ? el("p", { class: "tagline" }, film.tagline) : "",
      film.overview ? el("p", { class: "overview" }, film.overview) : "",
      film.collection ? el("div", { class: "dim" }, "Collection: " + film.collection.name) : "",
      film.cast.length > 0
        ? el(
            "div",
            { class: "cast" },
            el("span", { class: "dim" }, "Cast: "),
            film.cast
              .slice(0, 10)
              .map((c) => (c.character ? `${c.name} as ${c.character}` : c.name))
              .join(", ")
          )
        : ""
    )
  );
}

function certificationsLine(certs: Certifications | null, region: string): HTMLElement | null {
  const rating = certs?.[region];
  if (rating == null) return null;
  const line = el("div", { class: "certifications" });
  const icon = certificationIcon(region, rating);
  if (icon != null) {
    line.append(
      el("img", {
        class: "cert-icon",
        src: icon,
        alt: "",
        onerror: (e: Event) => {
          (e.currentTarget as HTMLImageElement).style.display = "none";
        },
      })
    );
  }
  line.append(`${region} ${rating}`);
  return line;
}

function renderActions(page: FilmPage): HTMLElement {
  const film = page.film;
  return el(
    "div",
    { class: "film-actions" },
    el(
      "button",
      {
        class: "btn",
        onclick: () =>
          void mutate(() => api.setFavorite(film.film_id, !film.favorite)),
      },
      film.favorite ? "★ Favorite" : "☆ Favorite"
    ),
    el(
      "button",
      {
        class: film.watchlisted ? "btn" : "btn primary",
        onclick: () =>
          void mutate(() => api.setWatchlisted(film.film_id, !film.watchlisted)),
      },
      film.watchlisted ? "Remove from Bucket List" : "Add to Bucket List"
    ),
    el(
      "button",
      {
        class: "btn",
        onclick: () => void mutate(() => api.refreshFilm(film.film_id)),
      },
      "Refresh metadata"
    ),
    el(
      "button",
      { class: "btn primary", onclick: () => startAddForFilm(film) },
      "Log another viewing"
    )
  );
}

function renderStats(page: FilmPage): HTMLElement {
  const stats = page.stats;
  const formats = Object.entries(stats.formats).sort((a, b) => b[1] - a[1]);
  return el(
    "div",
    { class: "film-stats" },
    el("h3", {}, "Viewings"),
    el(
      "div",
      { class: "dim" },
      `${stats.viewings} total · ${stats.theatre} theatre · ${stats.home} home`
    ),
    boundsLine(stats),
    formats.length > 0
      ? el(
          "div",
          { class: "chips" },
          ...formats.map(([label, count]) =>
            el("span", { class: "chip" }, `${label} × ${count}`)
          )
        )
      : ""
  );
}

function boundsLine(stats: FilmStats): HTMLElement | null {
  const bounds: string[] = [];
  if (stats.first_watched_at != null) bounds.push(`First ${formatDisplay(stats.first_watched_at)}`);
  if (stats.last_watched_at != null) bounds.push(`Last ${formatDisplay(stats.last_watched_at)}`);
  return bounds.length > 0 ? el("div", { class: "dim" }, bounds.join(" · ")) : null;
}

function renderHistory(page: FilmPage): HTMLElement {
  if (page.logs.length === 0) {
    return el("p", { class: "empty" }, "No viewings recorded.");
  }
  return el(
    "div",
    { class: "film-history" },
    el("h3", {}, "History"),
    el(
      "ul",
      { class: "log-list" },
      ...page.logs.map((log) => {
        const family = formatFamilyBadge(log.format);
        return el(
          "li",
          { class: "log-row" },
          el(
            "div",
            { class: "log-main" },
            el(
              "div",
              { class: "log-title" },
              formatDisplay(log.watched_at, log.timezone),
              log.rewatch ? el("span", { class: "badge rewatch" }, "Rewatch") : "",
              log.rating
                ? el("span", { class: "badge" }, log.rating === "up" ? "👍 Liked" : "👎 Disliked")
                : ""
            ),
            el(
              "div",
              { class: "log-sub" },
              family,
              family ? " " : "",
              el("span", { class: "dim" }, log.format_summary || "—")
            ),
            formatFlagsRow(log.format)
          )
        );
      })
    )
  );
}
