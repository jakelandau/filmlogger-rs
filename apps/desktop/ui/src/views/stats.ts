// Aggregate statistics over all stored logs: cards, the runtime line, pie
// charts for ratings and projection family, the switchable year/month/weekday
// timeline, the client-side time-of-day buckets, and the format/genre
// breakdowns. Inline CSS bars and conic gradients, no chart library.
import { el } from "../dom";
import { getState } from "../app";
import { timeOfDayCounts, weekdayCounts } from "../stats-time";
import type { FormatCatalog, FormatOption, Stats } from "../types";

/** Slice colors, shared with the Android stats screen. */
const FAMILY_COLORS: Record<string, string> = {
  dci: "#d489a4",
  imax: "#8fc4e8",
  film: "#ffb27a",
  home: "#8fe3c0",
};
const RATING_UP_COLOR = "#8fe3c0";
const RATING_DOWN_COLOR = "#e5484d";
const FALLBACK_COLOR = "#9d95a3";

type Slice = { label: string; count: number; color: string };

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

  content.append(
    pieSection("Ratings", ratingSlices(stats)),
    timelineSection(stats),
    orderedBreakdown("Viewings by time of day", timeOfDayCounts(getState().logs)),
    pieSection("Viewings by projection family", familySlices(stats)),
    breakdown("Home viewings by medium", stats.viewings_by_medium, (key) =>
      optionLabel(catalog?.mediums ?? null, key)
    ),
    breakdown("Viewings by presentation flag", stats.viewings_by_flag, (key) =>
      optionLabel(catalog?.flag_labels ?? null, key)
    ),
    orderedBreakdown("Top genres", topEntries(stats.viewings_by_genre, 10))
  );
}

/** `Time watched: …`, omitted when no runtime is known. */
function metaLine(stats: Stats): string {
  if (stats.total_viewings === 0 || stats.total_runtime_minutes === 0) return "";
  return `Time watched: ${formatDuration(stats.total_runtime_minutes)}`;
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

/** Liked/disliked slices; both are listed once anything is rated. */
function ratingSlices(stats: Stats): Slice[] {
  if (stats.ratings.up + stats.ratings.down === 0) return [];
  return [
    { label: "👍 Liked", count: stats.ratings.up, color: RATING_UP_COLOR },
    { label: "👎 Disliked", count: stats.ratings.down, color: RATING_DOWN_COLOR },
  ];
}

/** One slice per projection family, count descending then key ascending. */
function familySlices(stats: Stats): Slice[] {
  return Object.entries(stats.viewings_by_projection_family)
    .sort((a, b) => b[1] - a[1] || a[0].localeCompare(b[0]))
    .map(([key, count]) => ({
      label: familyLabel(key),
      count,
      color: FAMILY_COLORS[key] ?? FALLBACK_COLOR,
    }));
}

type TimelineMode = "year" | "month" | "weekday";

const TIMELINE_LABELS: Record<TimelineMode, string> = {
  year: "Year",
  month: "Month",
  weekday: "Weekday",
};

const TIMELINE_ORDER: TimelineMode[] = ["year", "month", "weekday"];

/** Year, month, and weekday viewings in one section, newest first. */
function timelineSection(stats: Stats): HTMLElement | string {
  const views: Record<TimelineMode, [string, number][]> = {
    year: Object.entries(stats.viewings_by_year)
      .map(([year, count]): [string, number] => [year, count])
      .sort((a, b) => b[0].localeCompare(a[0])),
    // The core orders months oldest first; show newest first like years.
    month: [...stats.viewings_by_month]
      .reverse()
      .map((month): [string, number] => [month.month, month.count]),
    weekday: weekdayCounts(getState().logs),
  };
  if (TIMELINE_ORDER.every((mode) => views[mode].length === 0)) return "";

  let mode: TimelineMode = "year";
  const body = el("div", { class: "breakdown-rows" });
  const buttons = TIMELINE_ORDER.map((key) =>
    el(
      "button",
      {
        class: "chip" + (key === mode ? " active" : ""),
        type: "button",
        onclick: () => {
          mode = key;
          for (const button of buttons) {
            button.classList.toggle("active", button.textContent === TIMELINE_LABELS[mode]);
          }
          body.replaceChildren(...barRows(views[mode]));
        },
      },
      TIMELINE_LABELS[key]
    )
  );
  body.append(...barRows(views[mode]));
  return el(
    "div",
    { class: "breakdown" },
    el("h3", {}, "Viewings over time"),
    el("div", { class: "chips" }, ...buttons),
    body
  );
}

/** A pie plus legend; omitted when there are no slices. */
function pieSection(title: string, slices: Slice[]): HTMLElement | string {
  const total = slices.reduce((sum, slice) => sum + slice.count, 0);
  if (total === 0) return "";
  let start = 0;
  const stops: string[] = [];
  for (const slice of slices) {
    if (slice.count === 0) continue;
    const end = start + (slice.count / total) * 100;
    stops.push(`${slice.color} ${start}% ${end}%`);
    start = end;
  }
  return el(
    "div",
    { class: "breakdown" },
    el("h3", {}, title),
    el(
      "div",
      { class: "pie-row" },
      el("div", {
        class: "pie",
        style: `background: conic-gradient(${stops.join(", ")})`,
      }),
      el(
        "div",
        { class: "pie-legend" },
        ...slices.map((slice) =>
          el(
            "div",
            { class: "legend-row" },
            el("span", {
              class: "legend-swatch",
              style: `background: ${slice.color}`,
            }),
            el("span", { class: "legend-label" }, slice.label),
            el("span", { class: "legend-count" }, String(slice.count))
          )
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
  return el(
    "div",
    { class: "breakdown" },
    el("h3", {}, title),
    el("div", { class: "breakdown-rows" }, ...barRows(entries))
  );
}

/** One proportional-bar row per entry, scaled to the largest count. */
function barRows(entries: [string, number][]): HTMLElement[] {
  const max = Math.max(...entries.map(([, count]) => count));
  return entries.map(([label, count]) =>
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
