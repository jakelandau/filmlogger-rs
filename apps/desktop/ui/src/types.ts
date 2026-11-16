// IPC boundary types. These mirror the Rust `serde` shapes of the
// `film-logger` domain exactly; the Rust side is the source of truth.

export type Venue = "theatre" | "home";
export type Rating = "up" | "down";
export interface RatingBoard {
  region: string;
  name: string;
}

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

export type Certifications = Record<string, string>;

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
  timezone: string | null;
  notes: string | null;
  rating: Rating | null;
  rewatch: boolean;
  watch_count: number | null;
  format: Format | null;
  format_summary: string;
}

export interface Settings {
  selected_board: string;
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
  viewings_by_month: Record<string, number>;
  viewings_by_projection_family: Record<string, number>;
  viewings_by_screen: Record<string, number>;
  viewings_by_medium: Record<string, number>;
  viewings_by_flag: Record<string, number>;
  rewatches: number;
  top_films: FilmViewings[];
}

export interface Genre {
  id: number;
  name: string;
}

export interface CastMember {
  id: number;
  name: string;
  character: string | null;
  order: number;
}

export interface CollectionRef {
  id: number;
  name: string;
  poster_path: string | null;
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
  cast: CastMember[];
  collection: CollectionRef | null;
}

export interface SearchPage {
  page: number;
  results: MovieSummary[];
  total_pages: number;
  total_results: number;
}

export interface FilmMetadata {
  film_id: string;
  tmdb_id: number | null;
  imdb_id: string | null;
  title: string;
  original_title: string | null;
  release_date: string | null;
  release_year: number | null;
  runtime_minutes: number | null;
  overview: string | null;
  tagline: string | null;
  original_language: string | null;
  poster_path: string | null;
  backdrop_path: string | null;
  vote_average: number | null;
  vote_count: number | null;
  popularity: number | null;
  adult: boolean;
  genres: Genre[];
  cast: CastMember[];
  certifications: Certifications | null;
  collection: CollectionRef | null;
  tags: string[];
  favorite: boolean;
  watchlisted: boolean;
  metadata_refreshed_at: string | null;
}

export interface FilmListEntry {
  film: FilmMetadata;
  viewings: number;
  last_watched_at: string | null;
}

export interface FilmStats {
  viewings: number;
  theatre: number;
  home: number;
  first_watched_at: string | null;
  last_watched_at: string | null;
  formats: Record<string, number>;
}

export interface FilmPage {
  film: FilmMetadata;
  logs: WatchLog[];
  stats: FilmStats;
}

export interface NewLogDetails {
  watched_at: string | null;
  timezone: string | null;
  notes: string | null;
  rating: Rating | null;
}

export interface FormatOption {
  key: string;
  label: string;
}

export interface FormatLegality {
  family: string;
  technology: string;
  screen: string;
  allowed_flags: string[];
  forced_flags: string[];
}

export interface FormatCatalog {
  families: FormatOption[];
  technologies: Record<string, FormatOption[]>;
  screens: Record<string, FormatOption[]>;
  mediums: FormatOption[];
  flag_labels: FormatOption[];
  legal: FormatLegality[];
}
