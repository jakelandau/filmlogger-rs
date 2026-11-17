// Pure rules for the Films tab sections. Seen and Bucket List are disjoint:
// a film with viewings is seen, a watchlisted film without viewings is on the
// bucket list, and a stored film with neither is library-only (reachable only
// through a collection or TMDB search). Collections are listed from stored
// films and their parts come from TMDB.
import type { Collection, CollectionPart, FilmListEntry } from "./types";

export type FilmSection = "seen" | "bucket";

/** Library entries keyed by TMDB id; entries without one are dropped. */
export function filmLibraryByTmdb(films: FilmListEntry[]): Map<number, FilmListEntry> {
  const library = new Map<number, FilmListEntry>();
  for (const entry of films) {
    const tmdbId = entry.film.tmdb_id;
    if (tmdbId != null) library.set(tmdbId, entry);
  }
  return library;
}

/** Library films in `section`, narrowed by favorites (Seen only) and title. */
export function visibleLibraryFilms(
  films: FilmListEntry[],
  section: FilmSection,
  favoritesOnly: boolean,
  needle: string,
): FilmListEntry[] {
  const query = needle.trim().toLowerCase();
  return films.filter((entry) => {
    const seen = entry.viewings > 0;
    const inSection =
      section === "seen"
        ? seen && (!favoritesOnly || entry.film.favorite)
        : entry.film.watchlisted && !seen;
    if (!inSection) return false;
    return query === "" || entry.film.title.toLowerCase().includes(query);
  });
}

export function collectionSeenCount(
  collection: Collection,
  library: Map<number, FilmListEntry>,
): number {
  return collection.parts.filter((part) => (library.get(part.tmdb_id)?.viewings ?? 0) > 0).length;
}

/** A collection with no parts is never complete. */
export function isCollectionComplete(
  collection: Collection,
  library: Map<number, FilmListEntry>,
): boolean {
  return (
    collection.parts.length > 0 &&
    collectionSeenCount(collection, library) === collection.parts.length
  );
}

/** Parts neither seen nor on the bucket list. */
export function collectionMissingParts(
  collection: Collection,
  library: Map<number, FilmListEntry>,
): CollectionPart[] {
  return collection.parts.filter((part) => {
    const entry = library.get(part.tmdb_id);
    return entry == null || (entry.viewings === 0 && !entry.film.watchlisted);
  });
}
