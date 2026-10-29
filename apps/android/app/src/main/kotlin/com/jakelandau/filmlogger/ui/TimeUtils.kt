package com.jakelandau.filmlogger.ui

import java.time.Instant
import java.time.LocalDateTime
import java.time.ZoneId
import java.time.ZoneOffset
import java.time.format.DateTimeFormatter

/**
 * Conversions between the stored RFC 3339 UTC instants and the device's local
 * wall-clock, for display and for the edit field.
 */
object TimeUtils {
    private val localFormatter = DateTimeFormatter.ofPattern("yyyy-MM-dd HH:mm")
    private val zone: ZoneId = ZoneId.systemDefault()

    private fun parse(iso: String): java.time.ZonedDateTime? =
        try {
            Instant.parse(iso).atZone(zone)
        } catch (_: Exception) {
            null
        }

    /** An instant rendered in the local zone as `yyyy-MM-dd HH:mm`. */
    fun display(iso: String): String = parse(iso)?.format(localFormatter) ?: iso

    /** The UTC calendar year of an instant, or null if it cannot be parsed. */
    fun year(iso: String): Int? =
        try {
            Instant.parse(iso).atZone(ZoneOffset.UTC).year
        } catch (_: Exception) {
            null
        }

    /** An instant as the local `yyyy-MM-dd HH:mm` edit string. */
    fun toLocalInput(iso: String): String =
        parse(iso)?.let { it.toLocalDateTime().format(localFormatter) } ?: iso

    /** A local `yyyy-MM-dd HH:mm` edit string back to an RFC 3339 instant. */
    fun fromLocalInput(value: String): String =
        try {
            LocalDateTime.parse(value, localFormatter)
                .atZone(zone)
                .toInstant()
                .toString()
        } catch (_: Exception) {
            ""
        }
}

/** Extract the year from a TMDB `YYYY-MM-DD` date, or null if absent. */
fun yearFrom(date: String?): Int? =
    date?.takeIf { it.length >= 4 }?.let { it.substring(0, 4).toIntOrNull() }
