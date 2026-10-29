package com.jakelandau.filmlogger.ui

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material3.Checkbox
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
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
import uniffi.film_logger_ffi.Format
import uniffi.film_logger_ffi.Venue

/**
 * A self-contained viewing-format selector. It owns its part state, seeds it
 * from [initial] (falling back to [venue] defaults on a venue mismatch), and
 * reports every change up via [onFormat].
 */
@Composable
fun FormatPicker(
    venue: Venue,
    initial: Format?,
    onFormat: (Format) -> Unit,
) {
    var parts by remember { mutableStateOf(FormatText.decompose(initial, venue)) }
    val scope = rememberCoroutineScope()
    // Seed the reported state so the first composition emits nothing.
    var lastReported by remember { mutableStateOf(FormatText.build(FormatText.decompose(initial, venue))) }

    fun update(p: FormatParts) {
        parts = p
        val built = FormatText.build(parts)
        if (built != lastReported) {
            lastReported = built
            scope.launch { onFormat(built) }
        }
    }

    Column(
        modifier = Modifier
            .fillMaxWidth()
            .padding(start = 12.dp, top = 4.dp, bottom = 8.dp),
    ) {
        if (venue == Venue.HOME) {
            OptionField("Release medium", parts.medium, FormatText.mediumOptions) { key ->
                update(parts.copy(medium = key, mediumCustom = if (key == "custom") parts.mediumCustom else ""))
            }
            if (parts.medium == "custom") {
                OutlinedTextField(
                    value = parts.mediumCustom,
                    onValueChange = { update(parts.copy(mediumCustom = it)) },
                    label = { Text("Custom medium id") },
                )
            }
        } else {
            OptionField("Projection family", parts.family, FormatText.familyOptions) { key ->
                update(parts.copy(family = key, tech = FormatText.defaultTech(key), techCustom = ""))
            }
            OptionField("Technology", parts.tech, FormatText.techOptions(parts.family)) { key ->
                update(parts.copy(tech = key, techCustom = if (key == "custom") parts.techCustom else ""))
            }
            if (parts.tech == "custom") {
                OutlinedTextField(
                    value = parts.techCustom,
                    onValueChange = { update(parts.copy(techCustom = it)) },
                    label = { Text("Custom technology id") },
                )
            }
            OptionField("Screen", parts.screen, FormatText.screenOptions) { key ->
                update(parts.copy(screen = key, screenCustom = if (key == "custom") parts.screenCustom else ""))
            }
            if (parts.screen == "custom") {
                OutlinedTextField(
                    value = parts.screenCustom,
                    onValueChange = { update(parts.copy(screenCustom = it)) },
                    label = { Text("Custom screen id") },
                )
            }
            Row(verticalAlignment = Alignment.CenterVertically) {
                Checkbox(checked = parts.threeD, onCheckedChange = { update(parts.copy(threeD = it)) })
                Text("3D")
                Spacer(Modifier.width(8.dp))
                Checkbox(checked = parts.hfr, onCheckedChange = { update(parts.copy(hfr = it)) })
                Text("HFR")
                Spacer(Modifier.width(8.dp))
                Checkbox(checked = parts.dbox, onCheckedChange = { update(parts.copy(dbox = it)) })
                Text("D-BOX")
            }
            Spacer(Modifier.height(4.dp))
        }
    }
}
