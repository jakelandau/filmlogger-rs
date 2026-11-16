// Aggregate statistics over all stored logs, including the format
// breakdowns, with inline CSS bars (no chart library).
import { el } from "../dom";
import { getState } from "../app";
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
  content.append(
    breakdown(
      "Viewings by month",
      stats.viewings_by_month,
      (key) => key,
      (a, b) => b[0].localeCompare(a[0])
    ),
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
    )
  );

  const years = Object.entries(stats.viewings_by_year)
    .map(([year, count]) => [Number(year), count] as const)
    .sort((a, b) => b[0] - a[0]);
  if (years.length > 0) {
    content.append(el("h3", {}, "Viewings by year"));
    content.append(
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

  if (stats.top_films.length > 0) {
    content.append(el("h3", {}, "Most watched"));
    content.append(
      el(
        "table",
        { class: "stats-table" },
        el("thead", {}, el("tr", {}, el("th", {}, "Film"), el("th", {}, "Times"))),
        el(
          "tbody",
          {},
          ...stats.top_films.map((f) =>
            el("tr", {}, el("td", {}, f.title), el("td", {}, String(f.count)))
          )
        )
      )
    );
  }
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

function breakdown(
  title: string,
  map: Record<string, number>,
  label: (key: string) => string,
  sort: (a: [string, number], b: [string, number]) => number = (a, b) => b[1] - a[1]
): HTMLElement | string {
  const entries = Object.entries(map).sort(sort);
  if (entries.length === 0) return "";
  const max = Math.max(...entries.map(([, count]) => count));
  return el(
    "div",
    { class: "breakdown" },
    el("h3", {}, title),
    el(
      "div",
      { class: "breakdown-rows" },
      ...entries.map(([key, count]) =>
        el(
          "div",
          { class: "breakdown-row" },
          el("span", { class: "breakdown-label" }, label(key)),
          el(
            "span",
            { class: "bar-track" },
            el("span", {
              class: "bar-fill",
              style: `width: ${(count / max) * 100}%`,
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
