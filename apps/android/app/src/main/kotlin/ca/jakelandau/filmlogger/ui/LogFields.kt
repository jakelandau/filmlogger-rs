package ca.jakelandau.filmlogger.ui

import android.app.DatePickerDialog
import android.app.TimePickerDialog
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.RadioButton
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.key
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import java.time.Instant
import java.time.LocalDateTime
import uniffi.filmlogger_rs_ffi.Format
import uniffi.filmlogger_rs_ffi.FormatCatalog
import uniffi.filmlogger_rs_ffi.Rating
import uniffi.filmlogger_rs_ffi.Venue

/**
 * The venue, viewing-format, watched-at, rating, and notes controls shared by
 * the add and edit views. `rewatch` and `watch_count` are store-derived, so
 * they are not editable here.
 */
@Composable
fun LogFields(
    catalog: FormatCatalog,
    venue: Venue,
    onVenue: (Venue) -> Unit,
    format: Format?,
    onFormat: (Format) -> Unit,
    timezone: String,
    onTimezone: (String) -> Unit,
    watchedAtInput: String,
    onWatchedAt: (String) -> Unit,
    rating: Rating?,
    onRating: (Rating) -> Unit,
) {
    FormatFields(catalog, venue, onVenue, format, onFormat, showFormatHeading = true)
    Spacer(Modifier.height(8.dp))
    ViewingFields(timezone, onTimezone, watchedAtInput, onWatchedAt, rating, onRating)
}

@Composable
fun FormatFields(
    catalog: FormatCatalog,
    venue: Venue,
    onVenue: (Venue) -> Unit,
    format: Format?,
    onFormat: (Format) -> Unit,
    showFormatHeading: Boolean = false,
) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.Center,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        RadioChoice(venue == Venue.THEATRE, { onVenue(Venue.THEATRE) }, "Theatre")
        Spacer(Modifier.width(16.dp))
        RadioChoice(venue == Venue.HOME, { onVenue(Venue.HOME) }, "Home")
    }
    Spacer(Modifier.height(8.dp))

    if (showFormatHeading) {
        Text("Viewing format", style = MaterialTheme.typography.titleSmall)
        Spacer(Modifier.height(6.dp))
    }

    key(venue) {
        FormatPicker(catalog, venue, format, onFormat)
    }
}

@Composable
fun ViewingFields(
    timezone: String,
    onTimezone: (String) -> Unit,
    watchedAtInput: String,
    onWatchedAt: (String) -> Unit,
    rating: Rating?,
    onRating: (Rating) -> Unit,
) {
    Text(
        "Time zone",
        modifier = Modifier.fillMaxWidth(),
        textAlign = TextAlign.Center,
        style = MaterialTheme.typography.titleSmall,
    )
    Spacer(Modifier.height(6.dp))
    TimezonePicker(timezone, onTimezone)
    Spacer(Modifier.height(12.dp))

    Text("Watched at", style = MaterialTheme.typography.titleSmall)
    Spacer(Modifier.height(6.dp))
    Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
        DateTimePickerButton(
            value = watchedAtInput,
            timezone = timezone,
            modifier = Modifier.weight(1f),
            onChange = onWatchedAt,
        )
        Spacer(Modifier.width(10.dp))
        OutlinedButton(
            onClick = {
                onWatchedAt(
                    TimeUtils.toLocalInput(
                        Instant.now().toString(),
                        TimeUtils.zone(timezone),
                    )
                )
            },
        ) {
            Text("Now")
        }
    }
    Spacer(Modifier.height(12.dp))

    Text(
        "Rating",
        modifier = Modifier.fillMaxWidth(),
        textAlign = TextAlign.Center,
        style = MaterialTheme.typography.titleSmall,
    )
    Spacer(Modifier.height(4.dp))
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.Center,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        RadioChoice(rating == Rating.UP, { onRating(Rating.UP) }, "👍 Up")
        Spacer(Modifier.width(20.dp))
        RadioChoice(rating == Rating.DOWN, { onRating(Rating.DOWN) }, "👎 Down")
    }
}

@Composable
private fun DateTimePickerButton(
    value: String,
    timezone: String,
    modifier: Modifier = Modifier,
    onChange: (String) -> Unit,
) {
    val context = LocalContext.current
    val zone = TimeUtils.zone(timezone)
    val start = TimeUtils.parseLocalInput(value) ?: LocalDateTime.now(zone)
    OutlinedButton(
        onClick = {
            DatePickerDialog(
                context,
                { _, year, month, day ->
                    TimePickerDialog(
                        context,
                        { _, hour, minute ->
                            onChange(
                                TimeUtils.formatLocalInput(
                                    LocalDateTime.of(year, month + 1, day, hour, minute),
                                )
                            )
                        },
                        start.hour,
                        start.minute,
                        android.text.format.DateFormat.is24HourFormat(context),
                    ).show()
                },
                start.year,
                start.monthValue - 1,
                start.dayOfMonth,
            ).show()
        },
        modifier = modifier,
    ) {
        Text(
            value.ifEmpty { "Select date and time" },
            style = MaterialTheme.typography.bodyLarge,
        )
    }
}

@Composable
private fun TimezonePicker(timezone: String, onTimezone: (String) -> Unit) {
    var expanded by remember { mutableStateOf(false) }
    var query by remember { mutableStateOf("") }
    Box {
        OutlinedButton(
            onClick = {
                query = ""
                expanded = true
            },
            modifier = Modifier.fillMaxWidth(),
        ) {
            Text(timezone)
        }
        if (expanded) {
            val needle = query.trim()
            val zones = remember(needle) {
                if (needle.isEmpty()) {
                    TimeUtils.availableZones()
                } else {
                    TimeUtils.availableZones().filter {
                        it.contains(needle, ignoreCase = true)
                    }
                }
            }
            AlertDialog(
                onDismissRequest = { expanded = false },
                title = { Text("Time zone") },
                text = {
                    Column {
                        OutlinedTextField(
                            value = query,
                            onValueChange = { query = it },
                            label = { Text("Search") },
                            singleLine = true,
                            modifier = Modifier.fillMaxWidth(),
                        )
                        Spacer(Modifier.height(8.dp))
                        if (zones.isEmpty()) {
                            Text(
                                "No matching time zones",
                                color = MaterialTheme.colorScheme.onSurfaceVariant,
                            )
                        } else {
                            Column(
                                Modifier
                                    .heightIn(max = 360.dp)
                                    .verticalScroll(rememberScrollState()),
                            ) {
                                for (zone in zones) {
                                    Row(
                                        Modifier
                                            .fillMaxWidth()
                                            .clickable {
                                                onTimezone(zone)
                                                expanded = false
                                            }
                                            .padding(vertical = 10.dp),
                                    ) {
                                        Text(zone)
                                    }
                                }
                            }
                        }
                    }
                },
                confirmButton = {
                    TextButton(onClick = { expanded = false }) { Text("Cancel") }
                },
            )
        }
    }
}

/** A radio button whose label is part of the tap target. */
@Composable
private fun RadioChoice(selected: Boolean, onSelect: () -> Unit, label: String) {
    Row(
        verticalAlignment = Alignment.CenterVertically,
        modifier = Modifier.clickable(onClick = onSelect),
    ) {
        RadioButton(selected = selected, onClick = onSelect)
        Text(label)
    }
}
