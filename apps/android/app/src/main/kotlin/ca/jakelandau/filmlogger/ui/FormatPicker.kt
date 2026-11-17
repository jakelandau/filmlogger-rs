package ca.jakelandau.filmlogger.ui

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material3.Checkbox
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.launch
import uniffi.filmlogger_rs_ffi.Format
import uniffi.filmlogger_rs_ffi.FormatCatalog
import uniffi.filmlogger_rs_ffi.FormatGroup
import uniffi.filmlogger_rs_ffi.FormatLegality
import uniffi.filmlogger_rs_ffi.Venue

private fun options(group: FormatGroup?): List<Pair<String, String>> =
    group?.options?.map { it.key to it.label } ?: emptyList()

/**
 * Adds a synthetic option for a legacy custom value the catalog does not know
 * (from an older database), so its id is preserved and displayed.
 */
private fun withLegacy(
    options: List<Pair<String, String>>,
    key: String,
    custom: String,
): List<Pair<String, String>> =
    if (key == "custom" && custom.isNotBlank() && options.none { it.first == "custom" }) {
        options + ("custom" to custom)
    } else {
        options
    }

private fun legalityOf(catalog: FormatCatalog, parts: FormatParts): FormatLegality? =
    catalog.legal.firstOrNull {
        it.family == parts.family && it.technology == parts.tech && it.screen == parts.screen
    }

/** Clamp the presentation flags to those the selected combination allows. */
private fun normalizeFlags(parts: FormatParts, catalog: FormatCatalog): FormatParts {
    val allowedFlags: List<String>
    val forcedFlags: List<String>
    if (parts.venue == Venue.HOME) {
        val entry = catalog.homeLegal.firstOrNull { it.medium == parts.medium }
        allowedFlags = entry?.allowedFlags ?: emptyList()
        forcedFlags = entry?.forcedFlags ?: emptyList()
    } else {
        val entry = legalityOf(catalog, parts)
        allowedFlags = entry?.allowedFlags ?: emptyList()
        forcedFlags = entry?.forcedFlags ?: emptyList()
    }
    fun resolve(forced: Boolean, allowed: Boolean, on: Boolean): Boolean =
        if (forced) true else allowed && on
    return parts.copy(
        threeD = resolve(
            forcedFlags.contains("three_d"),
            allowedFlags.contains("three_d"),
            parts.threeD,
        ),
        hfr = resolve(
            forcedFlags.contains("hfr"),
            allowedFlags.contains("hfr"),
            parts.hfr,
        ),
        dbox = resolve(
            forcedFlags.contains("dbox"),
            allowedFlags.contains("dbox"),
            parts.dbox,
        ),
    )
}

/**
 * A catalog-driven viewing-format selector: family → technology → auditorium
 * for theatre, and a release medium for home. Legality (which auditoria a
 * family runs in, which flags a combination may or must carry) comes from the
 * Rust-generated catalog.
 */
@Composable
fun FormatPicker(
    catalog: FormatCatalog,
    venue: Venue,
    initial: Format?,
    onFormat: (Format) -> Unit,
) {
    var parts by remember(initial, venue) {
        mutableStateOf(normalizeFlags(FormatText.decompose(initial, venue), catalog))
    }
    val scope = rememberCoroutineScope()
    var lastReported by remember(initial, venue) {
        mutableStateOf(FormatText.build(FormatText.decompose(initial, venue)))
    }

    fun update(next: FormatParts) {
        val normalized = normalizeFlags(next, catalog)
        parts = normalized
        val built = FormatText.build(normalized)
        if (built != lastReported) {
            lastReported = built
            scope.launch { onFormat(built) }
        }
    }

    Column(Modifier.fillMaxWidth()) {
        if (venue == Venue.HOME) {
            OptionField(
                "Release medium",
                parts.medium,
                withLegacy(catalog.mediums.map { it.key to it.label }, parts.medium, parts.mediumCustom),
            ) { key ->
                update(parts.copy(medium = key, mediumCustom = if (key == "custom") parts.mediumCustom else ""))
            }
            val entry = catalog.homeLegal.firstOrNull { it.medium == parts.medium }
            Row(verticalAlignment = Alignment.CenterVertically) {
                if (entry == null) {
                    Text("no presentation options")
                } else {
                    catalog.flagLabels.forEach { flag ->
                        val allowed = entry.allowedFlags.contains(flag.key)
                        val forced = entry.forcedFlags.contains(flag.key)
                        if (allowed) {
                            val checked = when (flag.key) {
                                "three_d" -> parts.threeD
                                "hfr" -> parts.hfr
                                else -> parts.dbox
                            }
                            Checkbox(
                                checked = checked,
                                enabled = !forced,
                                onCheckedChange = { on ->
                                    update(
                                        when (flag.key) {
                                            "three_d" -> parts.copy(threeD = on)
                                            "hfr" -> parts.copy(hfr = on)
                                            else -> parts.copy(dbox = on)
                                        },
                                    )
                                },
                            )
                            Text(flag.label, style = MaterialTheme.typography.bodyMedium)
                            Spacer(Modifier.width(10.dp))
                        }
                    }
                }
            }
            Spacer(Modifier.height(4.dp))
        } else {
            val screens = catalog.screens.firstOrNull { it.key == parts.family }
            OptionField(
                "Projection family",
                parts.family,
                catalog.families.map { it.key to it.label },
            ) { key ->
                val nextTech = options(catalog.technologies.firstOrNull { it.key == key }).firstOrNull()?.first ?: ""
                val nextScreen = options(catalog.screens.firstOrNull { it.key == key }).firstOrNull()?.first ?: ""
                update(
                    parts.copy(
                        family = key,
                        tech = nextTech,
                        techCustom = "",
                        screen = nextScreen,
                        screenCustom = "",
                        threeD = false,
                        hfr = false,
                        dbox = false,
                    ),
                )
            }
            OptionField(
                "Technology",
                parts.tech,
                withLegacy(
                    catalog.technologies.firstOrNull { it.key == parts.family }?.options?.map { it.key to it.label }
                        ?: emptyList(),
                    parts.tech,
                    parts.techCustom,
                ),
            ) { key ->
                update(parts.copy(tech = key, techCustom = if (key == "custom") parts.techCustom else ""))
            }
            OptionField(
                "Auditorium",
                parts.screen,
                withLegacy(options(screens), parts.screen, parts.screenCustom),
            ) { key ->
                update(parts.copy(screen = key, screenCustom = if (key == "custom") parts.screenCustom else ""))
            }
            val entry = legalityOf(catalog, parts)
            Row(verticalAlignment = Alignment.CenterVertically) {
                if (entry == null) {
                    Text("no presentation options")
                } else {
                    catalog.flagLabels.forEach { flag ->
                        val allowed = entry.allowedFlags.contains(flag.key)
                        val forced = entry.forcedFlags.contains(flag.key)
                        if (allowed) {
                            val checked = when (flag.key) {
                                "three_d" -> parts.threeD
                                "hfr" -> parts.hfr
                                else -> parts.dbox
                            }
                            Checkbox(
                                checked = checked,
                                enabled = !forced,
                                onCheckedChange = { on ->
                                    update(
                                        when (flag.key) {
                                            "three_d" -> parts.copy(threeD = on)
                                            "hfr" -> parts.copy(hfr = on)
                                            else -> parts.copy(dbox = on)
                                        },
                                    )
                                },
                            )
                            Text(flag.label, style = MaterialTheme.typography.bodyMedium)
                            Spacer(Modifier.width(10.dp))
                        }
                    }
                }
            }
            Spacer(Modifier.height(4.dp))
        }
    }
}
