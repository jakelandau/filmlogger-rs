// IPC boundary types. These mirror the Rust `serde` shapes of the
// `film-logger` domain exactly; the Rust side is the source of truth.

export type Venue = "theatre" | "home";
export type Rating = "up" | "down";
export type RatingBoard = "mpa" | "bbfc" | "fsk";

// Serde enums with unit variants serialize to bare strings; the `Custom`
// variant serializes to `{"custom": "..."}`. So a leaf is `string |
// { custom: string }`.
export type Leaf = string | { custom: string };

// Projection: exactly one of dci / imax / film, holding the family's leaf.
export type Projection =
  | { dci: Leaf }
  | { imax: Leaf }
  | { film: Leaf };

export type ScreenClass = Leaf;
export type HomeMedium = Leaf;

export type Format =
  | {
      theatre: {
        projection: Projection;
        screen: ScreenClass;
        three_d: boolean;
        hfr: boolean;
        dbox: boolean;
      };
    }
  | { home: { medium: HomeMedium } };

export interface Certifications {
  us: string | null;
  gb: string | null;
  de: string | null;
}

export interface WatchLog {
  id: number;
  film_id: string;
  imdb_id: string | null;
  tmdb_id: number | null;
  title: string;
  release_year: number | null;
  poster_path: string | null;
  certifications: Certifications | null;
  venue: Venue;
  watched_at: string;
  notes: string | null;
  rating: Rating | null;
  rewatch: boolean;
  watch_count: number | null;
  format: Format | null;
}

export interface Settings {
  selected_board: RatingBoard;
}

export interface VenueCounts {
  theatre: number;
  home: number;
}

export interface FilmViewings {
  title: string;
  count: number;
}

export interface Stats {
  total_viewings: number;
  distinct_films: number;
  viewings_by_venue: VenueCounts;
  // BTreeMap<i32, usize> serializes to a JSON object with string keys.
  viewings_by_year: Record<string, number>;
  rewatches: number;
  top_films: FilmViewings[];
}

export interface Genre {
  id: number;
  name: string;
}

export interface MovieSummary {
  id: number;
  title: string;
  original_title: string;
  original_language: string;
  release_date: string | null;
  overview: string;
  genre_ids: number[];
  poster_path: string | null;
  backdrop_path: string | null;
  vote_average: number;
  vote_count: number;
  popularity: number;
  adult: boolean;
}

export interface Movie {
  id: number;
  title: string;
  original_title: string;
  original_language: string;
  release_date: string | null;
  runtime: number | null;
  genres: Genre[];
  overview: string;
  tagline: string | null;
  status: string;
  poster_path: string | null;
  backdrop_path: string | null;
  vote_average: number;
  vote_count: number;
  imdb_id: string | null;
  popularity: number;
  adult: boolean;
  certifications: Certifications | null;
}

export interface SearchPage {
  page: number;
  results: MovieSummary[];
  total_pages: number;
  total_results: number;
}
