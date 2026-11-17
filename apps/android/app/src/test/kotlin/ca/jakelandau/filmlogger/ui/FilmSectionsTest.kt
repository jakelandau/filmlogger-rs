package ca.jakelandau.filmlogger.ui

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.filmlogger_rs_ffi.Collection
import uniffi.filmlogger_rs_ffi.CollectionPart
import uniffi.filmlogger_rs_ffi.FilmId
import uniffi.filmlogger_rs_ffi.FilmListEntry
import uniffi.filmlogger_rs_ffi.LibraryFilm

class FilmSectionsTest {
    private fun libraryFilm(
        tmdbId: Long?,
        title: String,
        favorite: Boolean = false,
        watchlisted: Boolean = false,
    ): LibraryFilm = LibraryFilm(
        filmId = FilmId.Imdb("tt0000001"),
        tmdbId = tmdbId,
        title = title,
        releaseYear = null,
        posterPath = null,
        collection = null,
        favorite = favorite,
        watchlisted = watchlisted,
    )

    private fun entry(film: LibraryFilm, viewings: UInt = 0u): FilmListEntry =
        FilmListEntry(film, viewings, null)

    private fun part(tmdbId: Long, title: String): CollectionPart =
        CollectionPart(tmdbId, title, null, null, null)

    private fun collection(parts: List<CollectionPart>): Collection =
        Collection(10L, "A Collection", null, null, null, parts)

    private fun titles(films: List<FilmListEntry>): List<String> =
        films.map { it.film.title }

    @Test
    fun libraryMapKeysByTmdbIdAndDropsEntriesWithoutOne() {
        val withId = entry(libraryFilm(7L, "Seven"))
        val withoutId = entry(libraryFilm(null, "Local"))
        val library = filmLibraryByTmdb(listOf(withId, withoutId))
        assertEquals(listOf(7L), library.keys.toList())
        assertEquals(withId, library[7L])
    }

    @Test
    fun seenAndBucketListAreDisjoint() {
        val seenWatchlisted = entry(libraryFilm(1L, "Seen Watchlisted", watchlisted = true), 2u)
        val bucketOnly = entry(libraryFilm(2L, "Bucket Only", watchlisted = true))
        val libraryOnly = entry(libraryFilm(3L, "Library Only"))
        val favoriteSeen = entry(libraryFilm(4L, "Favorite Seen", favorite = true), 1u)
        val films = listOf(seenWatchlisted, bucketOnly, libraryOnly, favoriteSeen)

        assertEquals(
            listOf("Seen Watchlisted", "Favorite Seen"),
            titles(visibleLibraryFilms(films, FilmTab.SEEN, false, "")),
        )
        assertEquals(
            listOf("Bucket Only"),
            titles(visibleLibraryFilms(films, FilmTab.BUCKET_LIST, false, "")),
        )
        assertFalse(
            visibleLibraryFilms(films, FilmTab.SEEN, false, "").any { it.film.tmdbId == 3L },
        )
        assertFalse(
            visibleLibraryFilms(films, FilmTab.BUCKET_LIST, false, "").any { it.film.tmdbId == 3L },
        )
    }

    @Test
    fun favoritesOnlyAppliesToTheSeenSection() {
        val favoriteBucket = entry(
            libraryFilm(5L, "Favorite Bucket", favorite = true, watchlisted = true),
        )
        val films = listOf(
            entry(libraryFilm(1L, "Seen Watchlisted", watchlisted = true), 2u),
            entry(libraryFilm(2L, "Bucket Only", watchlisted = true)),
            entry(libraryFilm(4L, "Favorite Seen", favorite = true), 1u),
            favoriteBucket,
        )
        assertEquals(
            listOf("Favorite Seen"),
            titles(visibleLibraryFilms(films, FilmTab.SEEN, true, "")),
        )
        assertEquals(
            listOf("Bucket Only", "Favorite Bucket"),
            titles(visibleLibraryFilms(films, FilmTab.BUCKET_LIST, true, "")),
        )
    }

    @Test
    fun titleMatchIsATrimmedCaseInsensitiveSubstring() {
        val films = listOf(entry(libraryFilm(1L, "Seen Watchlisted", watchlisted = true), 2u))
        assertEquals(
            listOf("Seen Watchlisted"),
            titles(visibleLibraryFilms(films, FilmTab.SEEN, false, "  WATCH  ")),
        )
    }

    @Test
    fun collectionHelpersCountMissingParts() {
        val seen = entry(libraryFilm(101L, "Seen"), 1u)
        val bucket = entry(libraryFilm(102L, "Bucket", watchlisted = true))
        val libraryOnly = entry(libraryFilm(103L, "Library Only"))
        val library = filmLibraryByTmdb(listOf(seen, bucket, libraryOnly))
        val parts = listOf(
            part(101L, "Seen"),
            part(102L, "Bucket"),
            part(103L, "Library Only"),
            part(104L, "Absent"),
        )

        assertEquals(1, seenCount(collection(parts), library))
        assertEquals(
            listOf(103L, 104L),
            missingCollectionParts(collection(parts), library).map { it.tmdbId },
        )
        assertFalse(
            missingCollectionParts(collection(parts), library).any {
                it.tmdbId == 101L || it.tmdbId == 102L
            },
        )
    }

    @Test
    fun collectionCompleteness() {
        val seen = entry(libraryFilm(101L, "Seen"), 1u)
        val parts = listOf(part(101L, "Seen"), part(102L, "Bucket"))
        val library = filmLibraryByTmdb(listOf(seen, entry(libraryFilm(102L, "Bucket"))))

        assertFalse(isCollectionComplete(collection(emptyList()), library))
        assertFalse(isCollectionComplete(collection(parts), library))

        val allSeen = entry(libraryFilm(101L, "Seen"), 1u) to
            entry(libraryFilm(102L, "Bucket"), 1u)
        val seenLibrary = filmLibraryByTmdb(listOf(allSeen.first, allSeen.second))
        assertTrue(isCollectionComplete(collection(parts), seenLibrary))
    }
}
