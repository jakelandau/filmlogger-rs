package ca.jakelandau.filmlogger.ui

import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.film_logger_ffi.FilmId
import uniffi.film_logger_ffi.Venue
import uniffi.film_logger_ffi.WatchLog

/** Mirrors the desktop `stats-time.test.ts` fixtures exactly. */
class StatsTimeTest {
    private fun log(
        watchedAt: String,
        timezone: String? = "America/New_York",
    ) = WatchLog(
        id = 1,
        filmId = FilmId.Imdb("tt0000001"),
        imdbId = null,
        tmdbId = null,
        title = "Film",
        releaseYear = null,
        posterPath = null,
        certifications = null,
        venue = Venue.THEATRE,
        watchedAt = watchedAt,
        timezone = timezone,
        notes = null,
        rating = null,
        rewatch = false,
        watchCount = 1,
        format = null,
        formatSummary = "—",
    )

    // America/New_York, after the 2026-11-01 switch to EST (UTC-5).
    private val fixtures = listOf(
        log("2026-11-16T02:30:00Z"), // Sunday 21:30
        log("2026-11-16T02:45:00Z"), // Sunday 21:45
        log("2026-11-16T14:05:00Z"), // Monday 09:05
        log("2026-11-17T02:30:00Z"), // Monday 21:30
    )

    @Test
    fun weekdaysBucketInTheLogsZoneMondayFirst() {
        assertEquals(
            listOf("Monday" to 2, "Sunday" to 2),
            weekdayCounts(fixtures),
        )
    }

    @Test
    fun hoursBucketInTheLogsZoneAscending() {
        assertEquals(
            listOf("09:00" to 1, "21:00" to 3),
            hourCounts(fixtures),
        )
    }

    @Test
    fun unknownZoneFallsBackToTheDeviceZone() {
        assertEquals(1, weekdayCounts(listOf(log("2026-11-16T02:30:00Z", "Not/AZone"))).sumOf { it.second })
        assertEquals(1, hourCounts(listOf(log("2026-11-16T02:30:00Z", null))).size)
    }

    @Test
    fun unparseableTimestampsAreSkipped() {
        assertEquals(emptyList<Pair<String, Int>>(), weekdayCounts(listOf(log("not-a-date"))))
        assertEquals(emptyList<Pair<String, Int>>(), hourCounts(listOf(log("not-a-date"))))
    }
}
