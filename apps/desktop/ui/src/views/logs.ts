// The log list: every viewing, filterable and sortable, with edit/delete
// actions. Format labels come from the stored `format_summary`.
import { el } from "../dom";
import { getState, go } from "../app";
import { formatDisplay } from "../time";
import { formatFamilyBadge } from "../format-family";
import { formatFlagsRow } from "../format-flags";
import { applyFilter, defaultFilter, type LogFilter } from "../filters";
import { certificationIcon } from "../cert-icons";
import { icon } from "../icons";
import type { WatchLog } from "../types";

const IMAGE_BASE = "https://image.tmdb.org/t/p/w92";

let filter: LogFilter = { ...defaultFilter };
let filtersExpanded = false;

export function renderLogs(content: HTMLElement): void {
  const logs = getState().logs;
  if (logs.length === 0) {
    content.append(
      el(
        "p",
        { class: "empty" },
        "No films logged yet ",
        el("button", { class: "link", onclick: () => go("add") }, "Add your first film")
      )
    );
    return;
  }

  content.append(renderFilterBar());

  const visible = applyFilter(logs, filter);
  if (visible.length === 0) {
    content.append(el("p", { class: "empty" }, "No viewings match these filters."));
    return;
  }

  const list = el("ul", { class: "log-list" });
  for (const log of visible) list.append(renderRow(log));
  content.append(list);
}

function renderAgain(): void {
  const content = document.querySelector<HTMLElement>(".content");
  if (content) {
    content.replaceChildren();
    renderLogs(content);
  }
}

function renderFilterBar(): HTMLElement {
  const years = [...new Set(getState().logs.map((log) => new Date(log.watched_at).getUTCFullYear()))]
    .sort((a, b) => b - a);

  const text = el("input", {
    class: "search",
    type: "text",
    placeholder: "Search logs",
    value: filter.text,
    oninput: (e: Event) => {
      filter = { ...filter, text: (e.currentTarget as HTMLInputElement).value };
      renderAgain();
    },
  });

  const filterButton = el(
    "button",
    {
      class:
        "icon-button" +
        (filtersExpanded || filter.venue !== "all" || filter.rating !== "all" ? " active" : ""),
      title: "Filters",
      "aria-label": "Filters",
      onclick: () => {
        filtersExpanded = !filtersExpanded;
        renderAgain();
      },
    },
    icon("filter")
  );

  const year = el(
    "select",
    {
      onchange: (e: Event) => {
        const value = (e.currentTarget as HTMLSelectElement).value;
        filter = { ...filter, year: value === "all" ? "all" : Number(value) };
        renderAgain();
      },
    },
    el("option", { value: "all" }, "All years"),
    ...years.map((y) => el("option", { value: String(y) }, String(y)))
  );
  year.value = String(filter.year);

  const sort = el(
    "select",
    {
      onchange: (e: Event) => {
        filter = {
          ...filter,
          sort: (e.currentTarget as HTMLSelectElement).value as LogFilter["sort"],
        };
        renderAgain();
      },
    },
    el("option", { value: "recent" }, "Most recent"),
    el("option", { value: "oldest" }, "Oldest first"),
    el("option", { value: "title" }, "Title"),
    el("option", { value: "rating" }, "Rating")
  );
  sort.value = filter.sort;

  const venueChips = el("div", { class: "chips filter-chips" });
  for (const [value, label] of [
    ["all", "All venues"],
    ["theatre", "Theatre"],
    ["home", "Home"],
  ] as const) {
    venueChips.append(
      el(
        "button",
        {
          class: filter.venue === value ? "chip active" : "chip",
          onclick: () => {
            filter = { ...filter, venue: value };
            renderAgain();
          },
        },
        label
      )
    );
  }

  const ratingChips = el("div", { class: "chips filter-chips" });
  for (const [value, label] of [
    ["all", "Any rating"],
    ["up", "👍 Up"],
    ["down", "👎 Down"],
  ] as const) {
    ratingChips.append(
      el(
        "button",
        {
          class: filter.rating === value ? "chip active" : "chip",
          onclick: () => {
            filter = { ...filter, rating: value };
            renderAgain();
          },
        },
        label
      )
    );
  }

  const bar = el(
    "div",
    { class: "filter-bar logs-filter" },
    el("div", { class: "filter-top" }, text, filterButton),
    filtersExpanded ? el("div", { class: "filter-group" }, venueChips, ratingChips) : "",
    el(
      "div",
      { class: "filter-row" },
      el("div", { class: "filter-field" }, el("label", {}, "Year"), year),
      el("div", { class: "filter-field" }, el("label", {}, "Sort"), sort)
    )
  );
  return bar;
}

/// The film's certification for the user's selected rating board, or null
/// when unknown (the film has no certifications for that board).
function certificationFor(log: WatchLog, board: string): string | null {
  return log.certifications?.[board.toUpperCase()] ?? null;
}

function certificationBadge(region: string, cert: string | null): HTMLElement {
  const badge = el("span", { class: "badge" + (cert ? "" : " unrated") });
  if (cert == null) {
    badge.append("Unrated");
    return badge;
  }
  const icon = certificationIcon(region, cert);
  if (icon != null) {
    badge.append(
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
  badge.append(" " + cert);
  return badge;
}

function renderRow(log: WatchLog): HTMLElement {
  const rating =
    log.rating === "up" ? "👍 Liked" : log.rating === "down" ? "👎 Disliked" : null;
  const board = getState().settings.selected_board;
  const cert = certificationFor(log, board);
  const venue = log.venue === "theatre" ? "🎭 Theatre" : "🏠 Home";
  const poster = log.poster_path
    ? el("img", { class: "poster", src: IMAGE_BASE + log.poster_path, alt: "" })
    : el("div", { class: "poster placeholder" });
  const family = formatFamilyBadge(log.format);

  const actions = el(
    "span",
    { class: "row-actions" },
    el("button", { class: "btn", onclick: () => go("edit", log.id) }, "Edit")
  );

  return el(
    "li",
    { class: "log-row" },
    poster,
    el(
      "div",
      { class: "log-main" },
      el(
        "div",
        { class: "log-title" },
        log.title,
        log.release_year != null ? el("span", { class: "dim" }, ` (${log.release_year})`) : "",
        log.rewatch ? el("span", { class: "badge rewatch" }, "Rewatch") : ""
      ),
      el(
        "div",
        { class: "log-sub" },
        el("span", { class: "dim" }, formatDisplay(log.watched_at, log.timezone)),
        el("span", { class: "dim" }, " · "),
        el("span", { class: "badge venue-" + log.venue }, venue),
        el("span", { class: "dim" }, " · "),
        family,
        family ? " " : "",
        el("span", { class: "dim" }, log.format_summary || "—")
      ),
      formatFlagsRow(log.format),
      el(
        "div",
        { class: "log-meta" },
        certificationBadge(board, cert),
        rating ? el("span", { class: "rating" }, rating) : ""
      )
    ),
    actions
  );
}


