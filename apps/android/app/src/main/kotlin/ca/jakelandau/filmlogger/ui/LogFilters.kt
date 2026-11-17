package ca.jakelandau.filmlogger.ui

import java.time.Instant
import java.time.ZoneOffset
import uniffi.filmlogger_rs_ffi.Rating
import uniffi.filmlogger_rs_ffi.Venue
import uniffi.filmlogger_rs_ffi.WatchLog

/** Log-list sort order, matching the desktop list. */
internal enum class LogSort { RECENT, OLDEST, TITLE, RATING }

internal fun utcMillis(log: WatchLog): Long = Instant.parse(log.watchedAt).toEpochMilli()

internal fun utcYear(log: WatchLog): Int =
    Instant.parse(log.watchedAt).atZone(ZoneOffset.UTC).year

internal fun ratingRank(rating: Rating?): Int = when (rating) {
    Rating.UP -> 0
    Rating.DOWN -> 1
    null -> 2
}

/**
 * Filter and sort logs for display, matching the desktop list semantics:
 * text matches the title case-insensitively, the year is the UTC year, and
 * sort ties fall back to recency.
 */
internal fun filterLogs(
    logs: List<WatchLog>,
    text: String,
    venue: Venue?,
    year: Int?,
    rating: Rating?,
    sort: LogSort,
): List<WatchLog> {
    val needle = text.trim().lowercase()
    val matched = logs.filter { log ->
        (venue == null || log.venue == venue) &&
            (year == null || utcYear(log) == year) &&
            (rating == null || log.rating == rating) &&
            (needle.isEmpty() || log.title.lowercase().contains(needle))
    }
    val recent: Comparator<WatchLog> =
        compareByDescending<WatchLog> { utcMillis(it) }.thenByDescending { it.id }
    return when (sort) {
        LogSort.RECENT -> matched.sortedWith(recent)
        LogSort.OLDEST -> matched.sortedWith(recent.reversed())
        LogSort.TITLE -> matched.sortedWith(
            compareBy<WatchLog> { it.title.lowercase() }.then(recent),
        )
        LogSort.RATING -> matched.sortedWith(
            compareBy<WatchLog> { ratingRank(it.rating) }.then(recent),
        )
    }
}
