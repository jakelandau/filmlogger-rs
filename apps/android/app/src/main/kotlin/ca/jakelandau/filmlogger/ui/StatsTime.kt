package ca.jakelandau.filmlogger.ui

import java.time.Instant
import uniffi.film_logger_ffi.WatchLog

/**
 * Weekday and hour-of-day buckets for the Stats tab. The core aggregates
 * every other dimension in UTC, but weekday and hour must be read in the
 * log's own wall clock, so each client resolves the zone locally.
 */

/** Weekday labels, Monday first (the parity contract order). */
private val WEEKDAYS = listOf(
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
    "Sunday",
)

/** The log's zone, falling back to the device zone as display does. */
private fun zoneOf(log: WatchLog) = TimeUtils.zone(log.timezone ?: TimeUtils.systemZone().id)

/** Viewings per weekday, Monday first, empty buckets omitted. */
internal fun weekdayCounts(logs: List<WatchLog>): List<Pair<String, Int>> {
    val counts = IntArray(WEEKDAYS.size)
    for (log in logs) {
        val instant = runCatching { Instant.parse(log.watchedAt) }.getOrNull() ?: continue
        // dayOfWeek: Monday = 1, so subtract one for the Monday-first index.
        counts[instant.atZone(zoneOf(log)).dayOfWeek.value - 1] += 1
    }
    return WEEKDAYS.mapIndexedNotNull { index, label ->
        if (counts[index] == 0) null else label to counts[index]
    }
}

/** Viewings per hour (`HH:00`), ascending, empty buckets omitted. */
internal fun hourCounts(logs: List<WatchLog>): List<Pair<String, Int>> {
    val counts = IntArray(24)
    for (log in logs) {
        val instant = runCatching { Instant.parse(log.watchedAt) }.getOrNull() ?: continue
        counts[instant.atZone(zoneOf(log)).hour] += 1
    }
    return counts.indices.mapNotNull { hour ->
        if (counts[hour] == 0) null else String.format("%02d:00", hour) to counts[hour]
    }
}
