package ca.jakelandau.filmlogger.ui

import uniffi.filmlogger_rs_ffi.Collection
import uniffi.filmlogger_rs_ffi.CollectionPart
import uniffi.filmlogger_rs_ffi.FilmListEntry

/**
 * The Films tab sections. Seen and Bucket List are disjoint: a film with
 * viewings is seen, a watchlisted film without viewings is on the bucket
 * list, and a stored film with neither is library-only.
 */
internal enum class FilmTab { SEEN, BUCKET_LIST, COLLECTIONS }

/** Library entries keyed by TMDB id; entries without one are dropped. */
internal fun filmLibraryByTmdb(films: List<FilmListEntry>): Map<Long, FilmListEntry> =
    films.mapNotNull { entry -> entry.film.tmdbId?.let { it to entry } }.toMap()

/** Library films in `tab`, narrowed by favorites (Seen only) and title. */
internal fun visibleLibraryFilms(
    films: List<FilmListEntry>,
    tab: FilmTab,
    favoritesOnly: Boolean,
    needle: String,
): List<FilmListEntry> {
    val query = needle.trim().lowercase()
    return films.filter { entry ->
        val seen = entry.viewings > 0u
        val inSection = when (tab) {
            FilmTab.SEEN -> seen && (!favoritesOnly || entry.film.favorite)
            FilmTab.BUCKET_LIST -> entry.film.watchlisted && !seen
            FilmTab.COLLECTIONS -> true
        }
        inSection && (query.isEmpty() || entry.film.title.lowercase().contains(query))
    }
}

/** How many of the collection's films have been seen. */
internal fun seenCount(
    collection: Collection,
    libraryByTmdb: Map<Long, FilmListEntry>,
): Int = collection.parts.count { part ->
    val viewings = libraryByTmdb[part.tmdbId]?.viewings ?: 0u
    viewings > 0u
}

/** A collection with no parts is never complete. */
internal fun isCollectionComplete(
    collection: Collection,
    libraryByTmdb: Map<Long, FilmListEntry>,
): Boolean = collection.parts.isNotEmpty() &&
    seenCount(collection, libraryByTmdb) == collection.parts.size

/** Parts neither seen nor on the bucket list. */
internal fun missingCollectionParts(
    collection: Collection,
    libraryByTmdb: Map<Long, FilmListEntry>,
): List<CollectionPart> = collection.parts.filter { part ->
    val entry = libraryByTmdb[part.tmdbId]
    entry == null || (entry.viewings == 0u && !entry.film.watchlisted)
}
