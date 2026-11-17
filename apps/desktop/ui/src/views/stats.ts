// Aggregate statistics over all stored logs: cards, a meta line, the rating
// breakdown, the client-side weekday/hour buckets, and every format/genre
// breakdown. Inline CSS bars, no chart library.
import { el } from "../dom";
import { getState } from "../app";
import { hourCounts, weekdayCounts } from "../stats-time";
import { formatDisplay } from "../time";
import type { FormatCatalog, FormatOption, Stats } from "../types";

export function renderStats(content: HTMLElement): void {
  const stats = getState().stats;
  if (!stats) {
    content.append(el("p", { class: "empty" }, "Loading…"));
    return;
  }

  content.append(
    el(
      "div",
      { class: "stat-cards" },
      statCard(stats.total_viewings, "Total viewings"),
      statCard(stats.distinct_films, "Distinct films"),
      statCard(stats.viewings_by_venue.theatre, "Theatre"),
      statCard(stats.viewings_by_venue.home, "Home"),
      statCard(stats.rewatches, "Rewatches"),
      statCard(rewatchRate(stats), "Rewatch rate")
    )
  );

  const catalog = getState().catalog;
  const meta = metaLine(stats);
  if (meta !== "") content.append(el("p", { class: "dim" }, meta));

  const ratings = orderedBreakdown("Ratings", [
    ["👍 Liked", stats.ratings.up],
    ["👎 Disliked", stats.ratings.down],
    ["Unrated", stats.ratings.unrated],
  ]);

  content.append(
    ratings,
    orderedBreakdown("Viewings by month", descending(stats.viewings_by_month)),
    orderedBreakdown("Viewings by weekday", weekdayCounts(getState().logs)),
    orderedBreakdown("Viewings by hour", hourCounts(getState().logs)),
    yearTable(stats),
    breakdown("Viewings by projection family", stats.viewings_by_projection_family, (key) =>
      familyLabel(key)
    ),
    breakdown("Viewings by auditorium", stats.viewings_by_screen, (key) =>
      optionLabel(catalog === null ? null : Object.values(catalog.screens).flat(), key)
    ),
    breakdown("Home viewings by medium", stats.viewings_by_medium, (key) =>
      optionLabel(catalog?.mediums ?? null, key)
    ),
    breakdown("Viewings by presentation flag", stats.viewings_by_flag, (key) =>
      optionLabel(catalog?.flag_labels ?? null, key)
    ),
    orderedBreakdown("Top genres", topEntries(stats.viewings_by_genre, 10)),
    topFilmsTable(stats)
  );
}

/** `Time watched: … · First: … · Last: …`, each half omitted when unknown. */
function metaLine(stats: Stats): string {
  if (stats.total_viewings === 0) return "";
  const parts: string[] = [];
  if (stats.total_runtime_minutes > 0) {
    parts.push(`Time watched: ${formatDuration(stats.total_runtime_minutes)}`);
  }
  if (stats.first_viewed_at != null) {
    parts.push(`First: ${formatDisplay(stats.first_viewed_at)}`);
  }
  if (stats.last_viewed_at != null) {
    parts.push(`Last: ${formatDisplay(stats.last_viewed_at)}`);
  }
  return parts.join(" · ");
}

/** `Xh Ym`, `Xh` on the hour, `Ym` under an hour. */
function formatDuration(minutes: number): string {
  const hours = Math.floor(minutes / 60);
  const rest = minutes % 60;
  if (hours === 0) return `${rest}m`;
  if (rest === 0) return `${hours}h`;
  return `${hours}h ${rest}m`;
}

/** A map's entries, count descending then key ascending, capped at `limit`. */
function topEntries(map: Record<string, number>, limit: number): [string, number][] {
  return Object.entries(map)
    .sort((a, b) => b[1] - a[1] || a[0].localeCompare(b[0]))
    .slice(0, limit);
}

/** A map's entries keyed in descending order (month keys sort lexically). */
function descending(map: Record<string, number>): [string, number][] {
  return Object.entries(map).sort((a, b) => b[0].localeCompare(a[0]));
}

function yearTable(stats: Stats): HTMLElement | string {
  const years = Object.entries(stats.viewings_by_year)
    .map(([year, count]) => [Number(year), count] as const)
    .sort((a, b) => b[0] - a[0]);
  if (years.length === 0) return "";
  return el(
    "div",
    { class: "breakdown" },
    el("h3", {}, "Viewings by year"),
    el(
      "table",
      { class: "stats-table" },
      el("thead", {}, el("tr", {}, el("th", {}, "Year"), el("th", {}, "Viewings"))),
      el(
        "tbody",
        {},
        ...years.map(([year, count]) =>
          el("tr", {}, el("td", {}, String(year)), el("td", {}, String(count)))
        )
      )
    )
  );
}

function topFilmsTable(stats: Stats): HTMLElement | string {
  const films = stats.top_films.slice(0, 10);
  if (films.length === 0) return "";
  return el(
    "div",
    { class: "breakdown" },
    el("h3", {}, "Most watched"),
    el(
      "table",
      { class: "stats-table" },
      el("thead", {}, el("tr", {}, el("th", {}, "Film"), el("th", {}, "Times"))),
      el(
        "tbody",
        {},
        ...films.map((f) =>
          el("tr", {}, el("td", {}, f.title), el("td", {}, String(f.count)))
        )
      )
    )
  );
}

function familyLabel(key: string): string {
  switch (key) {
    case "dci":
      return "Digital (DCI)";
    case "imax":
      return "IMAX";
    case "film":
      return "Film";
    case "home":
      return "Home";
    default:
      return key;
  }
}

function optionLabel(options: FormatOption[] | null, key: string): string {
  return options?.find((option) => option.key === key)?.label ?? key;
}

/** A breakdown from a map, count descending then label ascending. */
function breakdown(
  title: string,
  map: Record<string, number>,
  label: (key: string) => string
): HTMLElement | string {
  return orderedBreakdown(
    title,
    Object.entries(map)
      .map(([key, count]): [string, number] => [label(key), count])
      .sort((a, b) => b[1] - a[1] || a[0].localeCompare(b[0]))
  );
}

/** A breakdown from already-ordered entries, keeping that order. */
function orderedBreakdown(title: string, entries: [string, number][]): HTMLElement | string {
  if (entries.length === 0) return "";
  const max = Math.max(...entries.map(([, count]) => count));
  return el(
    "div",
    { class: "breakdown" },
    el("h3", {}, title),
    el(
      "div",
      { class: "breakdown-rows" },
      ...entries.map(([label, count]) =>
        el(
          "div",
          { class: "breakdown-row" },
          el("span", { class: "breakdown-label" }, label),
          el(
            "span",
            { class: "bar-track" },
            el("span", {
              class: "bar-fill",
              style: `width: ${max === 0 ? 0 : (count / max) * 100}%`,
            })
          ),
          el("span", { class: "breakdown-count" }, String(count))
        )
      )
    )
  );
}

function statCard(value: string | number, label: string): HTMLElement {
  return el(
    "div",
    { class: "stat-card" },
    el("div", { class: "stat-value" }, String(value)),
    el("div", { class: "stat-label" }, label)
  );
}

function rewatchRate(stats: Stats): string {
  if (stats.total_viewings === 0) return "—";
  const pct = Math.round((stats.rewatches / stats.total_viewings) * 100);
  return pct + "%";
}
