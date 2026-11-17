// Weekday and hour-of-day buckets for the Stats tab. The core crate
// aggregates every other dimension in UTC, but weekday and hour must be read
// in the log's own wall clock, so each client resolves the zone locally.

import { systemTimeZone } from "./time";
import type { WatchLog } from "./types";

/** Weekday labels, Monday first (the contract order). */
const WEEKDAYS = [
  "Monday",
  "Tuesday",
  "Wednesday",
  "Thursday",
  "Friday",
  "Saturday",
  "Sunday",
];

/** `log.timezone` when `Intl` accepts it, else the device zone. */
function resolveZone(timezone: string | null | undefined): string {
  if (timezone != null && timezone !== "") {
    try {
      new Intl.DateTimeFormat("en-US", { timeZone: timezone }).format(0);
      return timezone;
    } catch {
      // Unknown zone: fall back to the device zone, as display does.
    }
  }
  return systemTimeZone();
}

/** The wall-clock weekday (0 = Monday) and hour, or `null` if unparseable. */
function wallClock(iso: string, timeZone: string): { weekday: number; hour: number } | null {
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) return null;
  try {
    const parts = Object.fromEntries(
      new Intl.DateTimeFormat("en-US", {
        timeZone,
        year: "numeric",
        month: "2-digit",
        day: "2-digit",
        hour: "2-digit",
        hourCycle: "h23",
      })
        .formatToParts(date)
        .filter((part) => part.type !== "literal")
        .map((part) => [part.type, part.value])
    );
    // `Date.UTC` of the wall date: only the weekday is read, so no zone math.
    const wall = new Date(
      Date.UTC(Number(parts.year), Number(parts.month) - 1, Number(parts.day))
    );
    if (Number.isNaN(wall.getTime())) return null;
    // `getUTCDay()`: 0 = Sunday, so shift to Monday-first.
    return { weekday: (wall.getUTCDay() + 6) % 7, hour: Number(parts.hour) };
  } catch {
    return null;
  }
}

/** Viewings per weekday, Monday first, empty buckets omitted. */
export function weekdayCounts(logs: WatchLog[]): [string, number][] {
  const counts = WEEKDAYS.map(() => 0);
  for (const log of logs) {
    const clock = wallClock(log.watched_at, resolveZone(log.timezone));
    if (clock == null) continue;
    counts[clock.weekday] += 1;
  }
  return WEEKDAYS.flatMap((label, index): [string, number][] =>
    counts[index] === 0 ? [] : [[label, counts[index]]]
  );
}

/** Viewings per hour (`HH:00`), ascending, empty buckets omitted. */
export function hourCounts(logs: WatchLog[]): [string, number][] {
  const counts = Array.from({ length: 24 }, () => 0);
  for (const log of logs) {
    const clock = wallClock(log.watched_at, resolveZone(log.timezone));
    if (clock == null) continue;
    counts[clock.hour] += 1;
  }
  return counts.flatMap((count, hour): [string, number][] =>
    count === 0 ? [] : [[`${String(hour).padStart(2, "0")}:00`, count]]
  );
}
