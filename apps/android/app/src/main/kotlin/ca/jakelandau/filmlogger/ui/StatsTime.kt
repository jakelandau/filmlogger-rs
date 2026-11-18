package ca.jakelandau.filmlogger.ui

import java.time.Instant
import uniffi.filmlogger_rs_ffi.WatchLog

/**
 * Weekday and time-of-day buckets for the Stats tab. The core aggregates
 * every other dimension in UTC, but weekday and wall-clock time must be read
 * in the log's own zone, so each client resolves the zone locally.
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

/** Time-of-day quadrants, midnight first (the parity contract order). */
private val TIME_OF_DAY = listOf(
    "00:00–06:00",
    "06:00–12:00",
    "12:00–18:00",
    "18:00–24:00",
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

/** Viewings per time-of-day quadrant, midnight first, empty buckets omitted. */
internal fun timeOfDayCounts(logs: List<WatchLog>): List<Pair<String, Int>> {
    val counts = IntArray(TIME_OF_DAY.size)
    for (log in logs) {
        val instant = runCatching { Instant.parse(log.watchedAt) }.getOrNull() ?: continue
        counts[instant.atZone(zoneOf(log)).hour / 6] += 1
    }
    return TIME_OF_DAY.mapIndexedNotNull { index, label ->
        if (counts[index] == 0) null else label to counts[index]
    }
}
