import { describe, expect, it } from "vitest";
import { hourCounts, weekdayCounts } from "./stats-time";
import type { WatchLog } from "./types";

// Shared fixtures with the Android `StatsTimeTest`: America/New_York, after
// the 2026-11-01 switch to EST (UTC-5).
function log(watchedAt: string, timezone: string | null = "America/New_York"): WatchLog {
  return {
    id: 1,
    film_id: "tt0000001",
    imdb_id: null,
    tmdb_id: null,
    title: "Film",
    release_year: null,
    poster_path: null,
    certifications: null,
    venue: "theatre",
    watched_at: watchedAt,
    timezone,
    notes: null,
    rating: null,
    rewatch: false,
    watch_count: 1,
    format: null,
    format_summary: "—",
  };
}

const FIXTURES = [
  log("2026-11-16T02:30:00Z"), // Sunday 21:30
  log("2026-11-16T02:45:00Z"), // Sunday 21:45
  log("2026-11-16T14:05:00Z"), // Monday 09:05
  log("2026-11-17T02:30:00Z"), // Monday 21:30
];

describe("stats time buckets", () => {
  it("buckets weekdays in the log's zone, Monday first", () => {
    expect(weekdayCounts(FIXTURES)).toEqual([
      ["Monday", 2],
      ["Sunday", 2],
    ]);
  });

  it("buckets hours in the log's zone, ascending", () => {
    expect(hourCounts(FIXTURES)).toEqual([
      ["09:00", 1],
      ["21:00", 3],
    ]);
  });

  it("falls back to the device zone for an unknown zone", () => {
    const counts = weekdayCounts([log("2026-11-16T02:30:00Z", "Not/AZone")]);
    expect(counts.reduce((sum, [, count]) => sum + count, 0)).toBe(1);
    expect(hourCounts([log("2026-11-16T02:30:00Z", null)])).toHaveLength(1);
  });

  it("skips unparseable timestamps", () => {
    expect(weekdayCounts([log("not-a-date")])).toEqual([]);
    expect(hourCounts([log("not-a-date")])).toEqual([]);
  });
});
