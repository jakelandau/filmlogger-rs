package ca.jakelandau.filmlogger.ui

import uniffi.film_logger_ffi.Dci
import uniffi.film_logger_ffi.Film
import uniffi.film_logger_ffi.Format
import uniffi.film_logger_ffi.HomeMedium
import uniffi.film_logger_ffi.Imax
import uniffi.film_logger_ffi.Projection
import uniffi.film_logger_ffi.ScreenClass
import uniffi.film_logger_ffi.Venue

/** Display label for the projection family; `null` when no format is stored. */
fun formatFamily(format: Format?): String? = when (format) {
    null -> null
    is Format.Home -> "HOME"
    is Format.Theatre -> when (format.projection) {
        is Projection.Dci -> "DCI"
        is Projection.Imax -> "IMAX"
        is Projection.Film -> "FILM"
    }
}

/**
 * The format-selection state, flattened to string keys so the catalog-driven
 * picker can be built from and decomposed to the FFI [`Format`] enum tree.
 *
 * The venue determines which fields are meaningful: home viewings use
 * [medium]/[mediumCustom]; theatre viewings use [family]/[tech]/[screen] and
 * the presentation flags.
 */
data class FormatParts(
    val venue: Venue,
    val family: String,
    val tech: String,
    val techCustom: String,
    val screen: String,
    val screenCustom: String,
    val medium: String,
    val mediumCustom: String,
    val threeD: Boolean,
    val hfr: Boolean,
    val dbox: Boolean,
) {
    companion object {
        fun forVenue(venue: Venue): FormatParts = FormatParts(
            venue = venue,
            family = "dci",
            tech = "2k_lamp",
            techCustom = "",
            screen = "regular",
            screenCustom = "",
            medium = "bluray",
            mediumCustom = "",
            threeD = false,
            hfr = false,
            dbox = false,
        )
    }
}

/** Builds and decomposes viewing formats from flattened string keys. */
object FormatText {
    // --- build enums from keys ---

    private fun dciVariant(key: String, custom: String): Dci = when (key) {
        "2k_lamp" -> Dci.Lamp2K
        "2k_laser" -> Dci.Laser2K
        "4k_lamp" -> Dci.Lamp4K
        "4k_laser" -> Dci.Laser4K
        else -> Dci.Custom(custom)
    }

    private fun imaxVariant(key: String, custom: String): Imax = when (key) {
        "2k_xenon" -> Imax.Xenon2K
        "4k_single_laser" -> Imax.Laser4KSingle
        "4k_dual_laser" -> Imax.Laser4KDual
        "70mm" -> Imax.Film70mm
        else -> Imax.Custom(custom)
    }

    private fun filmVariant(key: String, custom: String): Film = when (key) {
        "35mm" -> Film.Mm35
        "70mm" -> Film.Mm70
        else -> Film.Custom(custom)
    }

    private fun screenVariant(key: String, custom: String): ScreenClass = when (key) {
        "regular" -> ScreenClass.Regular
        "plf" -> ScreenClass.Plf
        "vip" -> ScreenClass.Vip
        "4dx" -> ScreenClass.FourDx
        "screenx" -> ScreenClass.ScreenX
        "imax_class" -> ScreenClass.ImaxClassic
        "imax_retrofit" -> ScreenClass.ImaxRetrofit
        "ultra_4dx" -> ScreenClass.Ultra4Dx
        else -> ScreenClass.Custom(custom)
    }

    private fun mediumVariant(key: String, custom: String): HomeMedium = when (key) {
        "dvd" -> HomeMedium.Dvd
        "hd_dvd" -> HomeMedium.HdDvd
        "bluray" -> HomeMedium.BluRay
        "uhd_bluray" -> HomeMedium.UhdBluRay
        "laserdisc" -> HomeMedium.LaserDisc
        "streaming" -> HomeMedium.Streaming
        "broadcast" -> HomeMedium.Broadcast
        "telecine" -> HomeMedium.Telecine
        else -> HomeMedium.Custom(custom)
    }

    private fun projectionFor(family: String, tech: String, techCustom: String): Projection =
        when (family) {
            "imax" -> Projection.Imax(imaxVariant(tech, techCustom))
            "film" -> Projection.Film(filmVariant(tech, techCustom))
            else -> Projection.Dci(dciVariant(tech, techCustom))
        }

    /** Build a [`Format`] for `parts.venue` from the flattened parts. */
    fun build(parts: FormatParts): Format =
        if (parts.venue == Venue.HOME) {
            Format.Home(mediumVariant(parts.medium, parts.mediumCustom))
        } else {
            Format.Theatre(
                projection = projectionFor(parts.family, parts.tech, parts.techCustom),
                screen = screenVariant(parts.screen, parts.screenCustom),
                threeD = parts.threeD,
                hfr = parts.hfr,
                dbox = parts.dbox,
            )
        }

    /**
     * Flatten an existing [`Format`] (or a venue default) into [FormatParts].
     * A format whose venue does not match [venue] yields that venue's defaults.
     */
    fun decompose(format: Format?, venue: Venue): FormatParts {
        if (venue == Venue.HOME) {
            val home = format as? Format.Home
            val (medium, mediumCustom) = home?.let { mediumParts(it.medium) } ?: ("bluray" to "")
            return FormatParts(
                venue = venue,
                family = "dci",
                tech = "2k_lamp",
                techCustom = "",
                screen = "regular",
                screenCustom = "",
                medium = medium,
                mediumCustom = mediumCustom,
                threeD = false,
                hfr = false,
                dbox = false,
            )
        }
        val t = format as? Format.Theatre
        val (family, tech, techCustom) = t?.let {
            val f = familyKey(it.projection)
            val (techKey, custom) = techParts(it.projection)
            Triple(f, techKey, custom)
        } ?: (Triple("dci", "2k_lamp", ""))
        val (screen, screenCustom) = t?.let { screenParts(it.screen) } ?: ("regular" to "")
        return FormatParts(
            venue = venue,
            family = family,
            tech = tech,
            techCustom = techCustom,
            screen = screen,
            screenCustom = screenCustom,
            medium = "bluray",
            mediumCustom = "",
            threeD = t?.threeD ?: false,
            hfr = t?.hfr ?: false,
            dbox = t?.dbox ?: false,
        )
    }

    private fun familyKey(projection: Projection): String = when (projection) {
        is Projection.Dci -> "dci"
        is Projection.Imax -> "imax"
        is Projection.Film -> "film"
    }

    private fun techParts(projection: Projection): Pair<String, String> = when (projection) {
        is Projection.Dci -> dciParts(projection.dci)
        is Projection.Imax -> imaxParts(projection.imax)
        is Projection.Film -> filmParts(projection.film)
    }

    private fun dciParts(dci: Dci): Pair<String, String> = when (dci) {
        Dci.Lamp2K -> "2k_lamp" to ""
        Dci.Laser2K -> "2k_laser" to ""
        Dci.Lamp4K -> "4k_lamp" to ""
        Dci.Laser4K -> "4k_laser" to ""
        is Dci.Custom -> "custom" to dci.id
    }

    private fun imaxParts(imax: Imax): Pair<String, String> = when (imax) {
        Imax.Xenon2K -> "2k_xenon" to ""
        Imax.Laser4KSingle -> "4k_single_laser" to ""
        Imax.Laser4KDual -> "4k_dual_laser" to ""
        Imax.Film70mm -> "70mm" to ""
        is Imax.Custom -> "custom" to imax.id
    }

    private fun filmParts(film: Film): Pair<String, String> = when (film) {
        Film.Mm35 -> "35mm" to ""
        Film.Mm70 -> "70mm" to ""
        is Film.Custom -> "custom" to film.id
    }

    private fun screenParts(screen: ScreenClass): Pair<String, String> = when (screen) {
        ScreenClass.Regular -> "regular" to ""
        ScreenClass.Plf -> "plf" to ""
        ScreenClass.Vip -> "vip" to ""
        ScreenClass.FourDx -> "4dx" to ""
        ScreenClass.ScreenX -> "screenx" to ""
        ScreenClass.ImaxClassic -> "imax_class" to ""
        ScreenClass.ImaxRetrofit -> "imax_retrofit" to ""
        ScreenClass.Ultra4Dx -> "ultra_4dx" to ""
        is ScreenClass.Custom -> "custom" to screen.id
    }

    private fun mediumParts(medium: HomeMedium): Pair<String, String> = when (medium) {
        HomeMedium.Dvd -> "dvd" to ""
        HomeMedium.HdDvd -> "hd_dvd" to ""
        HomeMedium.BluRay -> "bluray" to ""
        HomeMedium.UhdBluRay -> "uhd_bluray" to ""
        HomeMedium.LaserDisc -> "laserdisc" to ""
        HomeMedium.Streaming -> "streaming" to ""
        HomeMedium.Broadcast -> "broadcast" to ""
        HomeMedium.Telecine -> "telecine" to ""
        is HomeMedium.Custom -> "custom" to medium.id
    }
}
