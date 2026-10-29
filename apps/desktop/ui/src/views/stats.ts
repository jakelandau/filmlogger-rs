// Aggregate statistics over all stored logs.
import { el } from "../dom";
import { getState } from "../app";
import type { Stats } from "../types";

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
