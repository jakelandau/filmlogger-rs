import { describe, expect, it } from "vitest";
import { applyFilter, defaultFilter, type LogFilter } from "./filters";
import type { WatchLog } from "./types";

let nextId = 1;

function log(
  overrides: Partial<WatchLog> & { watched_at: string; title: string }
): WatchLog {
  return {
    id: nextId++,
    film_id: "tt0000001",
    imdb_id: "tt0000001",
    tmdb_id: 1,
    release_year: 2000,
    poster_path: null,
    certifications: null,
    venue: "theatre",
    timezone: null,
    notes: null,
    rating: null,
    rewatch: false,
    watch_count: 1,
    format: null,
    format_summary: "",
    ...overrides,
  } as WatchLog;
}

const filter = (overrides: Partial<LogFilter>): LogFilter => ({
  ...defaultFilter,
  ...overrides,
});

describe("applyFilter", () => {
  const logs = [
    log({
      title: "Alpha",
      watched_at: "2020-06-01T12:00:00Z",
      venue: "theatre",
      rating: "down",
      notes: "loud",
    }),
    log({
      title: "Beta",
      watched_at: "2021-06-01T12:00:00Z",
      venue: "home",
      rating: "up",
      notes: "quiet",
    }),
    log({
      title: "Gamma",
      watched_at: "2019-01-01T12:00:00Z",
      venue: "theatre",
      rating: null,
    }),
  ];

  it("matches text against the title only, case-insensitively", () => {
    expect(applyFilter(logs, filter({ text: "alp" })).map((l) => l.title)).toEqual([
      "Alpha",
    ]);
    // Notes are no longer part of the UI or the filter.
    expect(applyFilter(logs, filter({ text: "QUIET" }))).toEqual([]);
  });

  it("filters by venue, UTC year, and rating", () => {
    expect(
      applyFilter(logs, filter({ venue: "theatre" })).map((l) => l.title)
    ).toEqual(["Alpha", "Gamma"]);
    expect(applyFilter(logs, filter({ year: 2021 })).map((l) => l.title)).toEqual([
      "Beta",
    ]);
    expect(applyFilter(logs, filter({ rating: "down" })).map((l) => l.title)).toEqual([
      "Alpha",
    ]);
    expect(applyFilter(logs, filter({})).length).toBe(3);
  });

  it("sorts most recent and oldest first with id tie-breaks", () => {
    const same = [
      log({ title: "Same A", watched_at: "2021-01-01T00:00:00Z" }),
      log({ title: "Same B", watched_at: "2021-01-01T00:00:00Z" }),
    ];
    expect(applyFilter(logs, filter({ sort: "recent" })).map((l) => l.title)).toEqual([
      "Beta",
      "Alpha",
      "Gamma",
    ]);
    expect(applyFilter(logs, filter({ sort: "oldest" })).map((l) => l.title)).toEqual([
      "Gamma",
      "Alpha",
      "Beta",
    ]);
    expect(applyFilter(same, filter({ sort: "recent" })).map((l) => l.title)).toEqual([
      "Same B",
      "Same A",
    ]);
    expect(applyFilter(same, filter({ sort: "oldest" })).map((l) => l.title)).toEqual([
      "Same A",
      "Same B",
    ]);
  });

  it("sorts by title with a recency tie-break", () => {
    expect(applyFilter(logs, filter({ sort: "title" })).map((l) => l.title)).toEqual([
      "Alpha",
      "Beta",
      "Gamma",
    ]);
  });

  it("sorts ratings up, then down, then unrated, then by recency", () => {
    expect(
      applyFilter(logs, filter({ sort: "rating" })).map((l) => l.title)
    ).toEqual(["Beta", "Alpha", "Gamma"]);
  });
});
