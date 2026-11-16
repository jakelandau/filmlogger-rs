// Log-list filtering and sorting. Pure functions over the loaded logs, shared
// by the list view and its tests.
import type { Rating, Venue, WatchLog } from "./types";

export type VenueFilter = "all" | Venue;
export type RatingFilter = "all" | Rating;
export type YearFilter = "all" | number;
export type LogSort = "recent" | "oldest" | "title" | "rating";

export interface LogFilter {
  text: string;
  venue: VenueFilter;
  year: YearFilter;
  rating: RatingFilter;
  sort: LogSort;
}

export const defaultFilter: LogFilter = {
  text: "",
  venue: "all",
  year: "all",
  rating: "all",
  sort: "recent",
};

const ratingRank = (rating: Rating | null): number =>
  rating === "up" ? 0 : rating === "down" ? 1 : 2;

const watchedTime = (log: WatchLog): number => new Date(log.watched_at).getTime();

const utcYear = (log: WatchLog): number => new Date(log.watched_at).getUTCFullYear();

/** Filter and sort logs for display. */
export function applyFilter(logs: WatchLog[], filter: LogFilter): WatchLog[] {
  const needle = filter.text.trim().toLowerCase();
  const matched = logs.filter((log) => {
    if (filter.venue !== "all" && log.venue !== filter.venue) return false;
    if (filter.year !== "all" && utcYear(log) !== filter.year) return false;
    if (filter.rating !== "all" && log.rating !== filter.rating) return false;
    if (needle === "") return true;
    return log.title.toLowerCase().includes(needle);
  });

  const recent = (a: WatchLog, b: WatchLog) => {
    const delta = watchedTime(b) - watchedTime(a);
    return delta !== 0 ? delta : b.id - a.id;
  };

  switch (filter.sort) {
    case "oldest":
      matched.sort((a, b) => {
        const delta = watchedTime(a) - watchedTime(b);
        return delta !== 0 ? delta : a.id - b.id;
      });
      break;
    case "title":
      matched.sort((a, b) => {
        const byTitle = a.title.localeCompare(b.title);
        return byTitle !== 0 ? byTitle : recent(a, b);
      });
      break;
    case "rating":
      matched.sort((a, b) => {
        const byRating = ratingRank(a.rating) - ratingRank(b.rating);
        return byRating !== 0 ? byRating : recent(a, b);
      });
      break;
    case "recent":
      matched.sort(recent);
      break;
  }
  return matched;
}
