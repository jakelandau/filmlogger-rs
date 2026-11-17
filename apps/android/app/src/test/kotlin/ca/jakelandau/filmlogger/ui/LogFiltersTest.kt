package ca.jakelandau.filmlogger.ui

import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.filmlogger_rs_ffi.FilmId
import uniffi.filmlogger_rs_ffi.Rating
import uniffi.filmlogger_rs_ffi.Venue
import uniffi.filmlogger_rs_ffi.WatchLog

class LogFiltersTest {
    private var nextId = 1L

    private fun log(
        title: String,
        watchedAt: String,
        venue: Venue = Venue.THEATRE,
        rating: Rating? = null,
        notes: String? = null,
    ): WatchLog = WatchLog(
        id = nextId++,
        filmId = FilmId.Imdb("tt0000001"),
        imdbId = "tt0000001",
        tmdbId = 1L,
        title = title,
        releaseYear = 2000,
        posterPath = null,
        certifications = null,
        venue = venue,
        watchedAt = watchedAt,
        timezone = null,
        notes = notes,
        rating = rating,
        rewatch = false,
        watchCount = 1,
        format = null,
        formatSummary = "",
    )

    private fun logs(): List<WatchLog> = listOf(
        log("Alpha", "2020-06-01T12:00:00Z", Venue.THEATRE, Rating.DOWN, notes = "loud"),
        log("Beta", "2021-06-01T12:00:00Z", Venue.HOME, Rating.UP, notes = "quiet"),
        log("Gamma", "2019-01-01T12:00:00Z", Venue.THEATRE),
    )

    private fun titles(
        logs: List<WatchLog>,
        text: String = "",
        venue: Venue? = null,
        year: Int? = null,
        rating: Rating? = null,
        sort: LogSort = LogSort.RECENT,
    ): List<String> = filterLogs(logs, text, venue, year, rating, sort).map { it.title }

    @Test
    fun textMatchesTitleOnlyCaseInsensitively() {
        assertEquals(listOf("Alpha"), titles(logs(), text = "alp"))
        // Notes are not part of the filter.
        assertEquals(emptyList<String>(), titles(logs(), text = "QUIET"))
    }

    @Test
    fun filtersByVenueUtcYearAndRating() {
        val logs = logs()
        assertEquals(listOf("Alpha", "Gamma"), titles(logs, venue = Venue.THEATRE))
        assertEquals(listOf("Beta"), titles(logs, year = 2021))
        assertEquals(listOf("Alpha"), titles(logs, rating = Rating.DOWN))
        assertEquals(3, titles(logs).size)
    }

    @Test
    fun sortsMostRecentAndOldestFirstWithIdTieBreaks() {
        val logs = logs()
        assertEquals(listOf("Beta", "Alpha", "Gamma"), titles(logs, sort = LogSort.RECENT))
        assertEquals(listOf("Gamma", "Alpha", "Beta"), titles(logs, sort = LogSort.OLDEST))

        val same = listOf(
            log("Same A", "2021-01-01T00:00:00Z"),
            log("Same B", "2021-01-01T00:00:00Z"),
        )
        assertEquals(listOf("Same B", "Same A"), titles(same, sort = LogSort.RECENT))
        assertEquals(listOf("Same A", "Same B"), titles(same, sort = LogSort.OLDEST))
    }

    @Test
    fun sortsByTitleWithRecencyTieBreak() {
        assertEquals(
            listOf("Alpha", "Beta", "Gamma"),
            titles(logs(), sort = LogSort.TITLE),
        )
    }

    @Test
    fun sortsRatingsUpThenDownThenUnratedThenRecency() {
        assertEquals(
            listOf("Beta", "Alpha", "Gamma"),
            titles(logs(), sort = LogSort.RATING),
        )
    }
}
