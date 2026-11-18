// Weekday and time-of-day buckets for the Stats tab. The core crate
// aggregates every other dimension in UTC, but weekday and wall-clock time
// must be read in the log's own zone, so each client resolves the zone
// locally.

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

/** Time-of-day quadrants, midnight first (the contract order). */
const TIME_OF_DAY = ["00:00–06:00", "06:00–12:00", "12:00–18:00", "18:00–24:00"];

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

/** Viewings per time-of-day quadrant, midnight first, empty buckets omitted. */
export function timeOfDayCounts(logs: WatchLog[]): [string, number][] {
  const counts = TIME_OF_DAY.map(() => 0);
  for (const log of logs) {
    const clock = wallClock(log.watched_at, resolveZone(log.timezone));
    if (clock == null) continue;
    counts[Math.floor(clock.hour / 6)] += 1;
  }
  return TIME_OF_DAY.flatMap((label, index): [string, number][] =>
    counts[index] === 0 ? [] : [[label, counts[index]]]
  );
}
