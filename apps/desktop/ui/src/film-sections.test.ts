import { describe, expect, it } from "vitest";
import {
  collectionMissingParts,
  collectionSeenCount,
  filmLibraryByTmdb,
  isCollectionComplete,
  visibleLibraryFilms,
} from "./film-sections";
import type {
  Collection,
  CollectionPart,
  FilmListEntry,
  LibraryFilm,
} from "./types";

function libraryFilm(overrides: Partial<LibraryFilm>): LibraryFilm {
  return {
    film_id: "tt0000001",
    tmdb_id: 1,
    title: "Film",
    release_year: null,
    poster_path: null,
    collection: null,
    favorite: false,
    watchlisted: false,
    ...overrides,
  };
}

function entry(
  film: Partial<LibraryFilm> & { tmdb_id: number; title: string },
  viewings = 0,
): FilmListEntry {
  return { film: libraryFilm(film), viewings, last_watched_at: null };
}

function part(tmdbId: number, title: string, releaseYear: number | null = null): CollectionPart {
  return {
    tmdb_id: tmdbId,
    title,
    release_date: releaseYear != null ? `${releaseYear}-01-01` : null,
    release_year: releaseYear,
    poster_path: null,
  };
}

function collection(parts: CollectionPart[]): Collection {
  return {
    id: 10,
    name: "A Collection",
    overview: null,
    poster_path: null,
    backdrop_path: null,
    parts,
  };
}

const titles = (films: FilmListEntry[]): string[] => films.map((film) => film.film.title);

describe("filmLibraryByTmdb", () => {
  it("keys entries by TMDB id and drops entries without one", () => {
    const withId = entry({ tmdb_id: 7, title: "Seven" });
    const withoutId = entry({ tmdb_id: null as unknown as number, title: "Local" });
    const library = filmLibraryByTmdb([withId, withoutId]);
    expect([...library.keys()]).toEqual([7]);
    expect(library.get(7)).toBe(withId);
  });
});

describe("visibleLibraryFilms", () => {
  const seenWatchlisted = entry({ tmdb_id: 1, title: "Seen Watchlisted", watchlisted: true }, 2);
  const bucketOnly = entry({ tmdb_id: 2, title: "Bucket Only", watchlisted: true });
  const libraryOnly = entry({ tmdb_id: 3, title: "Library Only" });
  const favoriteSeen = entry({ tmdb_id: 4, title: "Favorite Seen", favorite: true }, 1);
  const films = [seenWatchlisted, bucketOnly, libraryOnly, favoriteSeen];

  it("lists a watchlisted seen film in Seen but not the bucket list", () => {
    expect(titles(visibleLibraryFilms(films, "seen", false, ""))).toEqual([
      "Seen Watchlisted",
      "Favorite Seen",
    ]);
    expect(titles(visibleLibraryFilms(films, "bucket", false, ""))).toEqual(["Bucket Only"]);
  });

  it("lists a watchlisted unseen film only in the bucket list", () => {
    expect(visibleLibraryFilms(films, "seen", false, "").map((f) => f.film.tmdb_id)).not.toContain(
      2,
    );
    expect(
      visibleLibraryFilms(films, "bucket", false, "").map((f) => f.film.tmdb_id),
    ).toContain(2);
  });

  it("lists a library-only film in neither section", () => {
    expect(visibleLibraryFilms(films, "seen", false, "").map((f) => f.film.tmdb_id)).not.toContain(
      3,
    );
    expect(
      visibleLibraryFilms(films, "bucket", false, "").map((f) => f.film.tmdb_id),
    ).not.toContain(3);
  });

  it("applies favoritesOnly to Seen only", () => {
    const favoriteBucket = entry({ tmdb_id: 5, title: "Favorite Bucket", favorite: true, watchlisted: true });
    const withFavoriteBucket = [...films, favoriteBucket];
    expect(titles(visibleLibraryFilms(withFavoriteBucket, "seen", true, ""))).toEqual([
      "Favorite Seen",
    ]);
    expect(titles(visibleLibraryFilms(withFavoriteBucket, "bucket", true, ""))).toEqual([
      "Bucket Only",
      "Favorite Bucket",
    ]);
  });

  it("matches the title as a case-insensitive substring of the trimmed needle", () => {
    expect(titles(visibleLibraryFilms(films, "seen", false, "  WATCH  "))).toEqual([
      "Seen Watchlisted",
    ]);
  });
});

describe("collection helpers", () => {
  const seen = entry({ tmdb_id: 101, title: "Seen" }, 1);
  const bucket = entry({ tmdb_id: 102, title: "Bucket", watchlisted: true });
  const libraryOnly = entry({ tmdb_id: 103, title: "Library Only" });
  const library = filmLibraryByTmdb([seen, bucket, libraryOnly]);
  const parts = [
    part(101, "Seen", 2005),
    part(102, "Bucket", 2008),
    part(103, "Library Only", 2012),
    part(104, "Absent", 2015),
  ];

  it("counts seen parts", () => {
    expect(collectionSeenCount(collection(parts), library)).toBe(1);
  });

  it("treats absent, unstored, and unwatchlisted unseen parts as missing", () => {
    expect(collectionMissingParts(collection(parts), library).map((p) => p.tmdb_id)).toEqual([
      103, 104,
    ]);
  });

  it("does not treat watchlisted unseen or seen parts as missing", () => {
    const missing = collectionMissingParts(collection(parts), library).map((p) => p.tmdb_id);
    expect(missing).not.toContain(102);
    expect(missing).not.toContain(101);
  });

  it("is never complete without parts", () => {
    expect(isCollectionComplete(collection([]), library)).toBe(false);
  });

  it("is incomplete while a part is unseen", () => {
    expect(isCollectionComplete(collection(parts), library)).toBe(false);
  });

  it("is complete when every part is seen", () => {
    const allSeen = [seen, bucket, libraryOnly].map((item) =>
      entry({ tmdb_id: item.film.tmdb_id as number, title: item.film.title }, 1),
    );
    expect(
      isCollectionComplete(
        collection(parts.slice(0, 3)),
        filmLibraryByTmdb(allSeen),
      ),
    ).toBe(true);
  });
});
