package ca.jakelandau.filmlogger.ui

import java.time.Instant
import java.time.LocalDateTime
import java.time.ZoneId
import java.time.ZoneOffset
import java.time.ZonedDateTime
import java.time.format.DateTimeFormatter

/**
 * Conversions between stored RFC 3339 UTC instants and wall-clock values in a
 * selected IANA time zone. `ZoneRules` resolves daylight-saving transitions.
 */
object TimeUtils {
    private val localFormatter = DateTimeFormatter.ofPattern("yyyy-MM-dd HH:mm")

    fun systemZone(): ZoneId = ZoneId.systemDefault()

    fun zone(id: String): ZoneId =
        runCatching { ZoneId.of(id) }.getOrElse { systemZone() }

    private val continentAreas = setOf(
        "Africa",
        "America",
        "Antarctica",
        "Arctic",
        "Asia",
        "Atlantic",
        "Australia",
        "Europe",
        "Indian",
        "Pacific",
    )

    private val zones: List<String> by lazy {
        ZoneId.getAvailableZoneIds()
            .asSequence()
            .filter { id ->
                id.contains('/') && id.substringBefore('/') in continentAreas
            }
            .sorted()
            .toList()
    }

    fun availableZones(): List<String> = zones

    private fun parse(iso: String, zone: ZoneId): ZonedDateTime? =
        try {
            Instant.parse(iso).atZone(zone)
        } catch (_: Exception) {
            null
        }

    /** An instant rendered in `zone` as `yyyy-MM-dd HH:mm`. */
    fun display(iso: String, zone: ZoneId = systemZone()): String =
        parse(iso, zone)?.format(localFormatter) ?: iso

    /** The UTC calendar year of an instant, or null if it cannot be parsed. */
    fun year(iso: String): Int? =
        try {
            Instant.parse(iso).atZone(ZoneOffset.UTC).year
        } catch (_: Exception) {
            null
        }

    /** An instant as the `yyyy-MM-dd HH:mm` edit string in `zone`. */
    fun toLocalInput(iso: String, zone: ZoneId = systemZone()): String =
        parse(iso, zone)?.toLocalDateTime()?.format(localFormatter) ?: ""

    /** Parse the `yyyy-MM-dd HH:mm` edit string, or null when invalid. */
    fun parseLocalInput(value: String): LocalDateTime? =
        try {
            LocalDateTime.parse(value, localFormatter)
        } catch (_: Exception) {
            null
        }

    /** Format a local date-time as the `yyyy-MM-dd HH:mm` edit string. */
    fun formatLocalInput(value: LocalDateTime): String = value.format(localFormatter)

    /** A `zone` wall time back to an RFC 3339 instant. Empty on a DST gap. */
    fun fromLocalInput(value: String, zone: ZoneId = systemZone()): String =
        try {
            val local = LocalDateTime.parse(value, localFormatter)
            val offsets = zone.rules.getValidOffsets(local)
            if (offsets.isEmpty()) {
                ""
            } else {
                ZonedDateTime.ofLocal(local, zone, offsets.first()).toInstant().toString()
            }
        } catch (_: Exception) {
            ""
        }
}

/** Extract the year from a TMDB `YYYY-MM-DD` date, or null if absent. */
fun yearFrom(date: String?): Int? =
    date?.takeIf { it.length >= 4 }?.let { it.substring(0, 4).toIntOrNull() }
